use lucy_tetris_rs::core::board::TetrisEngine;
use lucy_tetris_rs::core::ai::AIBot;
use std::time::Instant;
use std::io::{Write, stdout};

fn main() {
    println!("LUCY TETRIS RS - HEADLESS MODE");
    println!("--------------------------------");

    let seed = 42; // Fixed seed for reproducibility or random
    let mut engine = TetrisEngine::new(10, 20, seed);
    
    let mut pieces_placed = 0;
    let start_time = Instant::now();
    let mut last_print_time = Instant::now();

    loop {
        if engine.game_over {
            break;
        }

        // AI 思考最佳走法 (2-Ply Lookahead with Rayon)
        let best_move = AIBot::get_best_move(&engine);

        match best_move {
            Some(target) => {
                // 直接將盤面狀態修改為 AI 決定的落點
                engine.current_piece.shape = target.shape;
                engine.current_piece.x = target.x;
                engine.current_piece.y = target.drop_y;
                
                // 鎖定方塊並處理消行
                engine.lock_piece();
                pieces_placed += 1;
            }
            None => {
                // 無路可走
                engine.game_over = true;
            }
        }

        // 每 100 毫秒更新一次終端機顯示
        if last_print_time.elapsed().as_millis() > 100 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let pps = pieces_placed as f64 / elapsed;
            print!("\rScore: {} | Lines: {} | Level: {} | Pieces: {} | Speed: {:.1} pps", 
                engine.score, engine.lines, engine.level, pieces_placed, pps);
            stdout().flush().unwrap();
            last_print_time = Instant::now();
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let pps = pieces_placed as f64 / elapsed;
    println!("\n\n=== GAME OVER ===");
    println!("Total Time: {:.2} seconds", elapsed);
    println!("Final Score: {}", engine.score);
    println!("Lines Cleared: {}", engine.lines);
    println!("Pieces Placed: {}", pieces_placed);
    println!("Average Speed: {:.1} pieces/sec", pps);
}
