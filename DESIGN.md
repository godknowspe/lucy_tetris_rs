# Lucy Tetris (Rust Edition) 🦀🧱

這是一個將 Python 版 `lucy_tetris` 以 Rust 語言全面重寫的高效能專案。
Rust 版本的目標是追求**極致的效能、記憶體安全，以及平行運算能力**。我們將利用 Rust 的特性，將 AI 前瞻搜尋 (Lookahead) 的速度提升數十倍至百倍。

## 1. 核心架構與技術選型 (Architecture & Tech Stack)

*   **程式語言**: Rust (Edition 2021)
*   **UI 遊戲引擎**: `macroquad` (輕量級、支援跨平台編譯至 WebAssembly 的 2D 遊戲引擎，非常適合替代 Pygame)
*   **平行運算**: `rayon` (Data-parallelism library，用於 AI 多步搜尋時的平行計算)
*   **隨機數**: `rand` (支援 Seed 固定的亂數生成器)

### 目錄結構 (Directory Structure)
```text
lucy_tetris_rs/
├── Cargo.toml                  # Rust 依賴管理與設定檔
├── DESIGN.md                   # 系統設計文件
└── src/
    ├── main.rs                 # 程式進入點 (Game Loop)
    ├── core/                   # 核心邏輯引擎 (純 Rust，無外部 UI 依賴)
    │   ├── mod.rs
    │   ├── board.rs            # 遊戲盤面狀態 (考慮使用高效的 1D Array 或 Bitboard)
    │   ├── pieces.rs           # 方塊定義與旋轉系統 (SRS)
    │   ├── ai.rs               # 走法生成與 AI 搜尋 (結合 Rayon)
    │   └── simulator.rs        # 狀態克隆與虛擬沙盒模擬
    └── ui/                     # 遊戲介面
        ├── mod.rs
        └── renderer.rs         # 基於 Macroquad 的畫面渲染與事件綁定
```

---

## 2. 開發階段規劃 (Development Phases)

### Phase 1: 基礎引擎與 Macroquad UI (Core Engine & UI)
*   **目標**: 重新以 Rust 刻畫 Tetris 規則，支援手動遊玩。
*   **實作細節**:
    *   `pieces.rs`: 使用 `[[u8; 4]; 4]` 矩陣儲存方塊形狀，實作矩陣旋轉演算法。
    *   `board.rs`: 實作 `grid`，使用單一型別 (例如 `u8`) 代表顏色/方塊 ID。實作碰撞偵測 (`is_valid_position`) 與消行 (`clear_lines`)。
    *   `renderer.rs`: 使用 `macroquad::prelude::*` 畫出簡約風格的網格與方塊，綁定鍵盤輸入 (Left, Right, Down, Space)。

### Phase 2: 高效能模擬器與走法生成 (High-Performance Simulator)
*   **目標**: 實作 AI 思考的虛擬沙盒。
*   **實作細節**:
    *   因為 Rust 有嚴格的所有權 (Ownership) 限制，`simulator.rs` 將實作為接收 `&Board` 進行 Clone 以產生 `Simulator` 實例。
    *   `MoveGenerator` 將窮舉所有 `rotation` 與 `x` 座標，並返回一個包含所有合法落點的 `Vec<Move>`。

### Phase 3: 啟發式評估函數 (Heuristic Evaluator)
*   **目標**: 移植 Dellacherie 算分邏輯。
*   **實作細節**:
    *   在 `ai.rs` 中實作 `Evaluator` 結構體。
    *   利用 Rust 的 Iterator (如 `.iter().filter().count()`) 高效計算 `holes`, `aggregate_height`, `bumpiness`。

### Phase 4: 平行化 2-Ply 前瞻搜尋 (Parallel Lookahead AI)
*   **目標**: 釋放 Rust 的多核運算威力。
*   **實作細節**:
    *   引入 `rayon` crate。
    *   將第一步與第二步的巢狀迴圈改為 `.par_iter()`。讓 AI 在模擬 900+ 種盤面時，自動分配到所有 CPU 核心進行平行運算，達成毫秒級的 2-Ply 甚至 3-Ply 搜尋。
    *   實作 Headless 模式 (無 UI) 用於極速效能基準測試 (Benchmark)。
