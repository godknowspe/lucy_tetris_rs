# Lucy Tetris (Rust Edition) - Design Document 🦀

這是一個將 Python 版 `lucy_tetris` 以 Rust 語言全面重寫的高效能專案。
Rust 版本的目標是追求**極致的效能、記憶體安全，以及平行運算能力**。我們成功利用 Rust 的特性，將 AI 前瞻搜尋 (Lookahead) 的速度提升了上百倍 (達到 1500+ PPS)。

## 1. 核心架構與技術選型 (Architecture & Tech Stack)

* **程式語言**: Rust (Edition 2021)
* **UI 遊戲引擎**: `macroquad` (輕量級、支援非同步 `async` Game Loop 且不卡死運算的 2D 遊戲引擎)
* **平行運算**: `rayon` (Data-parallelism library，用於 2-Ply AI 多步搜尋的平行計算)
* **隨機數**: `rand` (具備 Deterministic 特性的 `StdRng`，支援 Fixed Seed)

### 目錄結構 (Directory Structure)
```text
lucy_tetris_rs/
├── Cargo.toml                  # 依賴與編譯設定
├── src/
│   ├── main.rs                 # UI 模式進入點
│   ├── lib.rs                  # 函式庫匯出 (供 headless 使用)
│   ├── bin/
│   │   └── headless.rs         # 極速無頭測試模式進入點
│   ├── core/                   # 核心邏輯引擎 (無 UI 依賴)
│   │   ├── board.rs            # 盤面、狀態機、碰撞與消行邏輯
│   │   ├── pieces.rs           # 方塊形狀與 3x3/4x4 旋轉邏輯
│   │   ├── simulator.rs        # 提供 AI 模擬的輕量化介面
│   │   └── ai.rs               # Rayon 2-Ply AI 與 Dellacherie 評估函數
│   └── ui/
│       └── renderer.rs         # Macroquad 渲染與事件狀態機
```

## 2. 效能最佳化設計 (Performance Optimizations)

### 2.1 零成本克隆 (Zero-cost Clone) 與避免 Heap Allocation
在 Python 版本中，使用 `copy.deepcopy` 複製盤面會產生大量的記憶體分配。在 Rust 中，盤面 (`Vec<Vec<u8>>`) 與方塊 (`[[u8; 4]; 4]`) 的 `.clone()` 會直接在底層進行連續記憶體區塊複製，速度極快。方塊的形狀直接寫死為 4x4 的 Stack Array，避免了任何額外的 Heap 負擔。

### 2.2 基於 Rayon 的 2-Ply 平行演算
在 `MoveGenerator` 列舉出當前方塊 (Ply 1) 的所有合法落點後，我們不使用單執行緒遍歷，而是使用 `moves_ply1.par_iter_mut().for_each(...)` 將未來第二顆方塊 (Next Piece, Ply 2) 的推演分配到所有可用的 CPU 核心上執行。由於 Rust 的 Ownership 模型，這保證了執行緒安全 (Thread-safe)，我們甚至不需要使用 `Mutex` 或 `RwLock` 就能完成狀態推演。

### 2.3 獨立的 Simulator
為了不干擾遊戲主體的 `TetrisEngine`，我們設計了獨立的 `Simulator`。它唯讀借用 (`&`) 基礎盤面，然後在自己的 Scope 內完成掉落、鎖定與消行的模擬，生命週期結束後立刻被釋放。

## 3. 遊戲邏輯修復與特徵 (Game Logic Specifics)

### 3.1 3x3 邊界框旋轉 (Bounding Box Rotation)
標準的 4x4 矩陣旋轉會導致 `T`, `J`, `L`, `S`, `Z` 等佔據 3 格的方塊發生「重心飄移」，甚至在靠牆時無法旋轉 (Wall-kick Fail)。Rust 實作中針對這些方塊，特別實作了「左上角 3x3 區塊」的獨立旋轉演算法，完美解決了手感異常的問題。

### 3.2 借用檢查器 (Borrow Checker) 與 UI 渲染
在 `update_playing` 迴圈中，我們曾遇到「可變借用 (mutable borrow)」修改引擎狀態，與「不可變借用 (immutable borrow)」渲染畫面之間的衝突。解決方案是利用區塊 `{}` 將 `engine.as_mut()` 的生命週期限制在資料更新階段，確保資料修改完成後才將控制權交還給唯讀的 `draw_game()` 方法。

## 4. AI 評估函數 (Heuristic Evaluation)

本專案實作了 Pierre Dellacherie 的經典啟發式演算法，並嚴格管控型別轉換 (`f32` 與 `i32`) 避免溢位。

* **Aggregated Height (總高度) [-0.51]**: 所有行高度的總和。高度越高，得分越低。
* **Completed Lines (消行數) [+0.76]**: 該步模擬能消除的行數。
* **Holes (洞穴數) [-0.36]**: 被方塊覆蓋的空格數量（最致命的缺點）。
* **Bumpiness (表面不平整度) [-0.18]**: 相鄰欄位高度差的絕對值總和。

## 5. 模式特徵 (Modes & Features)

### Headless Mode (無頭模式)
完全移除 `macroquad` 與視窗生命週期，專注於執行 AI 核心迴圈 (`AIBot::get_best_move`) 並直接修改記憶體盤面狀態。在此模式下系統能突破 1500 PPS 的速度，是非常理想的效能基準測試 (Benchmark) 與未來機器學習 (ML) 的訓練環境。

### Phase 5: MANUAL_AI (手動診斷模式)
為了解析 AI 決策，我們在 UI 中加入了 Diagnostics 面板。當玩家手動操控方塊時，系統會動態實例化一個 Simulator 預判玩家落點的分數，並同步顯示 AI 目標（紅色殘影）。當 `pieces_placed` 狀態改變時，強制重新生成 AI 目標，確保畫面的正確性與即時性。
