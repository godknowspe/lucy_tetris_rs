# Lucy Tetris RS 🦀🧱

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Macroquad](https://img.shields.io/badge/Macroquad-Game_Engine-blue?style=for-the-badge)
![Rayon](https://img.shields.io/badge/Rayon-Parallel_Computing-red?style=for-the-badge)

Lucy Tetris RS 是一個追求極致效能的俄羅斯方塊引擎與 AI 專案。這是從 Python 版本 (`lucy_tetris`) 移植並徹底重寫的高效能版本。透過 Rust 的記憶體安全設計、零成本抽象 (Zero-cost Abstractions) 以及 `rayon` 函式庫的資料平行處理，本專案在 2-Ply Lookahead AI 推演上達到了 **1500+ PPS (Pieces Per Second)** 的驚人運算速度。

## ✨ 核心特色 (Features)

* **極致效能的底層引擎**：捨棄 Heap Allocation，全盤使用 `[[u8; 4]; 4]` 固定長度陣列，狀態克隆 (Clone) 成本極低。
* **2-Ply 平行 AI 搜尋 (`Rayon`)**：同時計算當前方塊與下一個方塊 (Next Piece) 所有的合法排列組合，並於多執行緒中平行展開搜尋。
* **Dellacherie 啟發式演算法**：精準的權重評分系統 (Height: -0.51, Lines: +0.76, Holes: -0.36, Bumpiness: -0.18)。
* **無頭模式 (Headless Benchmark Mode)**：專門用於極限壓力測試與機器學習演算法評估的終端機模式。
* **四段式 AI 輔助系統**：結合 UI 視覺化，提供 `OFF`、`FAST`、`SLOW` 以及 `MANUAL_AI` 模式。
* **即時診斷面板 (Diagnostics Panel)**：在手動輔助模式下，即時計算玩家當前落點的評估分數，並用紅色殘影提示 AI 的最佳策略。

## 🚀 安裝與執行 (Installation & Usage)

請確保您的系統已安裝 Rust (透過 `rustup`)。

```bash
# Clone repository (若您已經有程式碼則省略)
git clone <repository-url>
cd lucy_tetris_rs

# 執行具備視覺化介面的遊戲模式
cargo run --release

# 執行極速無頭測試模式 (Headless Benchmark)
cargo run --release --bin headless
```

## 🎮 操作說明 (Controls)

### 選單介面 (Menu)
* **`Up` / `Down`**: 切換選單項目
* **`Left` / `Right`**: 調整版面寬度 / 高度
* **`A`**: 切換 AI 模式
* **`Enter`**: 開始遊戲

### 遊戲中 (In-Game)
* **`Left` / `Right`**: 左右平移方塊
* **`Down`**: 軟降下 (Soft Drop)
* **`Space`**: 旋轉方塊
* **`Up`**: 暫停遊戲 (Pause)
* **`A`**: 即時切換 AI 模式 (OFF -> FAST -> SLOW -> MANUAL_AI)

## 🤖 AI 模式詳解 (AI Modes)

按下 `A` 鍵可以循環切換以下模式：
1. **OFF**: 純手動遊玩。
2. **FAST**: AI 自動接管，無延遲瞬間落下 (Hard Drop)，展現極致運算速度。
3. **SLOW**: AI 自動接管，但會配合關卡速度慢慢落下，方便觀察 AI 的移動軌跡與補位策略。
4. **MANUAL_AI**: 玩家手動控制，但畫面會出現**紅色殘影**提示 AI 的最佳落點。右下角會展開 Diagnostics 診斷面板，即時計算並比較玩家當前落點與 AI 最佳落點的各項參數得分。
