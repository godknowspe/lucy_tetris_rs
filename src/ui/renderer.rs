use crate::core::ai::{AIBot, Move, Evaluator, EvalDetails};
use crate::core::simulator::Simulator;
use macroquad::prelude::*;
use crate::core::board::TetrisEngine;

pub enum GameState {
    Menu,
    Playing,
    GameOver,
}

pub enum Theme {
    Classic,
    SuperMario,
}

pub struct Renderer {
    state: GameState,
    theme: Theme,
    brick_tex: Option<macroquad::texture::Texture2D>,
    question_tex: Option<macroquad::texture::Texture2D>,
    config_width: usize,
    config_height: usize,
    menu_selection: u8,
    engine: Option<TetrisEngine>,
    fall_time: f32,
    fall_speed: f32,
    cell_size: f32,
    key_repeat_timer: f32,
    ai_mode: u8, // 0: OFF, 1: ON_FAST, 2: ON_SLOW, 3: MANUAL_AI
    ai_target_move: Option<Move>,
    ai_move_timer: f32,
    current_eval: Option<EvalDetails>,
    ai_last_pieces_placed: u32,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            state: GameState::Menu,
            theme: Theme::SuperMario, // Default to Super Mario theme
            brick_tex: None,
            question_tex: None,
            config_width: 10,
            config_height: 20,
            menu_selection: 0,
            engine: None,
            fall_time: 0.0,
            fall_speed: 0.5,
            cell_size: 30.0,
            key_repeat_timer: 0.0,
            ai_mode: 0,
            ai_target_move: None,
            ai_move_timer: 0.0,
            current_eval: None,
            ai_last_pieces_placed: 0,
        }
    }

    pub async fn run(&mut self) {
        // 載入材質
        if let Ok(tex) = load_texture("src/ui/pic/brick.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.brick_tex = Some(tex);
        }
        if let Ok(tex) = load_texture("src/ui/pic/question.png").await {
            tex.set_filter(FilterMode::Nearest);
            self.question_tex = Some(tex);
        }

        loop {
            clear_background(if matches!(self.theme, Theme::SuperMario) { Color::new(0.36, 0.58, 0.98, 1.0) /* Sky Blue */ } else { Color::new(0.12, 0.12, 0.12, 1.0) });

            match self.state {
                GameState::Menu => self.update_menu(),
                GameState::Playing => self.update_playing(),
                GameState::GameOver => self.update_gameover(),
            }

            next_frame().await
        }
    }

    fn update_menu(&mut self) {
        if is_key_pressed(KeyCode::Up) {
            self.menu_selection = (self.menu_selection + 3) % 4;
        } else if is_key_pressed(KeyCode::Down) {
            self.menu_selection = (self.menu_selection + 1) % 4;
        }

        if is_key_pressed(KeyCode::Left) {
            if self.menu_selection == 0 && self.config_width > 5 { self.config_width -= 1; }
            if self.menu_selection == 1 && self.config_height > 10 { self.config_height -= 1; }
        } else if is_key_pressed(KeyCode::Right) {
            if self.menu_selection == 0 && self.config_width < 30 { self.config_width += 1; }
            if self.menu_selection == 1 && self.config_height < 40 { self.config_height += 1; }
        } else if is_key_pressed(KeyCode::A) {
            self.ai_mode = (self.ai_mode + 1) % 4;
        } else if is_key_pressed(KeyCode::Enter) {
            if self.menu_selection == 3 {
                self.start_game();
            } else {
                self.menu_selection = (self.menu_selection + 1) % 4;
            }
        }

        let title = "LUCY TETRIS (RUST)";
        draw_text(title, screen_width()/2.0 - 150.0, 100.0, 40.0, WHITE);

        let ai_str = match self.ai_mode { 0 => "OFF", 1 => "FAST", 2 => "SLOW", _ => "MANUAL_AI" };
        let items = [
            format!("Board Width: < {} >", self.config_width),
            format!("Board Height: < {} >", self.config_height),
            format!("AI Mode (Press 'A'): {}", ai_str),
            "START GAME".to_string()
        ];

        for (i, text) in items.iter().enumerate() {
            let color = if i as u8 == self.menu_selection { YELLOW } else { WHITE };
            draw_text(text, screen_width()/2.0 - 100.0, 250.0 + i as f32 * 50.0, 30.0, color);
        }
    }

    fn start_game(&mut self) {
        let seed = (get_time() * 1000.0) as u64;
        self.engine = Some(TetrisEngine::new(self.config_width, self.config_height, seed));
        self.state = GameState::Playing;
        self.fall_time = 0.0;
        self.ai_target_move = None;
        self.current_eval = None;
        self.ai_last_pieces_placed = 0;

        let max_board_w = screen_width() - 350.0; // slightly more space for diag panel
        let max_board_h = screen_height() - 40.0;
        let cell_w = max_board_w / self.config_width as f32;
        let cell_h = max_board_h / self.config_height as f32;
        self.cell_size = 30.0_f32.min(cell_w).min(cell_h);
    }

    fn update_playing(&mut self) {
        let mut is_game_over = false;
        
        if let Some(engine) = self.engine.as_mut() {
            if is_key_pressed(KeyCode::Up) {
                engine.toggle_pause();
            }
            if is_key_pressed(KeyCode::A) {
                self.ai_mode = (self.ai_mode + 1) % 4;
                self.ai_target_move = None;
            }

            if !engine.paused && !engine.game_over {
                let dt = get_frame_time();
                self.fall_time += dt;

                let current_speed = (self.fall_speed - (engine.level as f32 - 1.0) * 0.05).max(0.1);
                
                if self.fall_time >= current_speed {
                    engine.move_piece(0, 1);
                    self.fall_time = 0.0;
                }
                
                // Check if new piece spawned
                if engine.pieces_placed != self.ai_last_pieces_placed {
                    self.ai_target_move = None;
                    self.ai_last_pieces_placed = engine.pieces_placed;
                }

                // If any AI is on (including MANUAL_AI), calculate the target
                if self.ai_mode != 0 {
                    self.ai_move_timer += dt;
                    if self.ai_move_timer > 0.05 {
                        self.ai_move_timer = 0.0;
                        if self.ai_target_move.is_none() {
                            self.ai_target_move = AIBot::get_best_move(engine);
                        }
                        
                        // ONLY AUTO EXECUTE in FAST(1) or SLOW(2) modes
                        if self.ai_mode == 1 || self.ai_mode == 2 {
                            if let Some(target) = &self.ai_target_move {
                                if engine.current_piece.shape != target.shape {
                                    engine.rotate_piece();
                                } else if engine.current_piece.x < target.x {
                                    engine.move_piece(1, 0);
                                } else if engine.current_piece.x > target.x {
                                    engine.move_piece(-1, 0);
                                } else if self.ai_mode == 1 {
                                    // FAST MODE: Hard drop
                                    let mut hit_bottom = false;
                                    while !hit_bottom {
                                        if !engine.move_piece(0, 1) { hit_bottom = true; }
                                    }
                                    self.ai_target_move = None;
                                }
                            }
                        }
                    }
                }

                if is_key_pressed(KeyCode::Space) { engine.rotate_piece(); }
                
                let mut dx = 0;
                let mut dy = 0;
                
                if is_key_pressed(KeyCode::Left) { dx = -1; self.key_repeat_timer = 0.0;}
                if is_key_pressed(KeyCode::Right) { dx = 1; self.key_repeat_timer = 0.0;}
                if is_key_pressed(KeyCode::Down) { dy = 1; self.key_repeat_timer = 0.0;}

                if is_key_down(KeyCode::Left) || is_key_down(KeyCode::Right) || is_key_down(KeyCode::Down) {
                    self.key_repeat_timer += dt;
                    if self.key_repeat_timer > 0.15 {
                        if is_key_down(KeyCode::Left) { dx = -1; }
                        if is_key_down(KeyCode::Right) { dx = 1; }
                        if is_key_down(KeyCode::Down) { dy = 1; }
                        self.key_repeat_timer = 0.1;
                    }
                }

                let piece_before = engine.current_piece.shape_id;
                let mut manual_moved = false;
                if dx != 0 || dy != 0 {
                    engine.move_piece(dx, dy);
                    manual_moved = true;
                }
                if piece_before != engine.current_piece.shape_id || manual_moved {
                    if self.ai_mode == 1 || self.ai_mode == 2 {
                        self.ai_target_move = None;
                    }
                }

                // If MANUAL_AI mode, evaluate the current ghost position
                if self.ai_mode == 3 {
                    let mut sim = Simulator::new(engine.width, engine.height, &engine.grid);
                    if sim.is_valid_position(&engine.current_piece, engine.current_piece.x, engine.current_piece.y) {
                        let drop_y = sim.drop_piece(&engine.current_piece, engine.current_piece.x, engine.current_piece.y);
                        sim.lock_piece(&engine.current_piece, engine.current_piece.x, drop_y);
                        let cleared = sim.clear_lines();
                        self.current_eval = Some(Evaluator::evaluate_details(&sim.grid, cleared));
                    }
                }
            }
            is_game_over = engine.game_over;
        }

        self.draw_game();

        if is_game_over {
            self.state = GameState::GameOver;
        }
    }

    fn update_gameover(&mut self) {
        self.draw_game();
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.6));
        draw_text("GAME OVER", screen_width()/2.0 - 100.0, screen_height()/2.0, 50.0, RED);
        draw_text("Press ENTER to return", screen_width()/2.0 - 120.0, screen_height()/2.0 + 40.0, 30.0, WHITE);
        if is_key_pressed(KeyCode::Enter) {
            self.state = GameState::Menu;
        }
    }

    fn draw_game(&self) {
        let engine = self.engine.as_ref().unwrap();
        let offset_x = 20.0;
        let offset_y = 20.0;

        let colors = [
            BLACK, 
            Color::new(0.0, 1.0, 1.0, 1.0), // I
            Color::new(0.0, 0.0, 1.0, 1.0), // J
            Color::new(1.0, 0.5, 0.0, 1.0), // L
            Color::new(1.0, 1.0, 0.0, 1.0), // O
            Color::new(0.0, 1.0, 0.0, 1.0), // S
            Color::new(0.5, 0.0, 0.5, 1.0), // T
            Color::new(1.0, 0.0, 0.0, 1.0), // Z
        ];

        // Draw grid
        let grid_col = if matches!(self.theme, Theme::SuperMario) { Color::new(1.0, 1.0, 1.0, 0.3) } else { DARKGRAY };
        for y in 0..engine.height {
            for x in 0..engine.width {
                let rect_x = offset_x + x as f32 * self.cell_size;
                let rect_y = offset_y + y as f32 * self.cell_size;
                draw_rectangle_lines(rect_x, rect_y, self.cell_size, self.cell_size, 1.0, grid_col);
                
                let cell = engine.grid[y][x];
                if cell != 0 {
                    let is_question = (cell & 128) != 0;
                    let color_id = (cell & 0x7F) as usize;
                    
                    if matches!(self.theme, Theme::SuperMario) && self.brick_tex.is_some() {
                        let tex = if is_question && self.question_tex.is_some() {
                            self.question_tex.as_ref().unwrap()
                        } else {
                            self.brick_tex.as_ref().unwrap()
                        };
                        draw_texture_ex(
                            tex,
                            rect_x, rect_y,
                            WHITE, // Use WHITE to disable color tinting
                            DrawTextureParams {
                                dest_size: Some(vec2(self.cell_size, self.cell_size)),
                                ..Default::default()
                            }
                        );
                    } else {
                        draw_rectangle(rect_x + 1.0, rect_y + 1.0, self.cell_size - 2.0, self.cell_size - 2.0, colors[color_id]);
                    }
                }
            }
        }

        // Draw current piece ghost
        let sim = Simulator::new(engine.width, engine.height, &engine.grid);
        if sim.is_valid_position(&engine.current_piece, engine.current_piece.x, engine.current_piece.y) {
            let drop_y = sim.drop_piece(&engine.current_piece, engine.current_piece.x, engine.current_piece.y);
            let ghost_color = Color::new(1.0, 1.0, 1.0, 0.2); // White transparent
            for r in 0..4 {
                for c in 0..4 {
                    if engine.current_piece.shape[r][c] != 0 {
                        let y = drop_y + r as i32;
                        let x = engine.current_piece.x + c as i32;
                        if y >= 0 {
                            let rect_x = offset_x + x as f32 * self.cell_size;
                            let rect_y = offset_y + y as f32 * self.cell_size;
                            draw_rectangle_lines(rect_x, rect_y, self.cell_size, self.cell_size, 2.0, ghost_color);
                        }
                    }
                }
            }
        }

        // Draw AI target ghost (red transparent) in MANUAL_AI mode
        if self.ai_mode == 3 {
            if let Some(target) = &self.ai_target_move {
                let ai_ghost_col = Color::new(1.0, 0.0, 0.0, 0.4);
                for r in 0..4 {
                    for c in 0..4 {
                        if target.shape[r][c] != 0 {
                            let y = target.drop_y + r as i32;
                            let x = target.x + c as i32;
                            if y >= 0 {
                                let rect_x = offset_x + x as f32 * self.cell_size;
                                let rect_y = offset_y + y as f32 * self.cell_size;
                                draw_rectangle(rect_x + 1.0, rect_y + 1.0, self.cell_size - 2.0, self.cell_size - 2.0, ai_ghost_col);
                                draw_rectangle_lines(rect_x, rect_y, self.cell_size, self.cell_size, 1.0, RED);
                            }
                        }
                    }
                }
            }
        }

        // Draw current piece
        for r in 0..4 {
            for c in 0..4 {
                if engine.current_piece.shape[r][c] != 0 {
                    let y = engine.current_piece.y + r as i32;
                    let x = engine.current_piece.x + c as i32;
                    if y >= 0 {
                        let rect_x = offset_x + x as f32 * self.cell_size;
                        let rect_y = offset_y + y as f32 * self.cell_size;
                        let shape_id = engine.current_piece.shape_id;
                        let is_question = (shape_id & 128) != 0;
                        let color_id = (shape_id & 0x7F) as usize;
                        
                        if matches!(self.theme, Theme::SuperMario) && self.brick_tex.is_some() {
                            let tex = if is_question && self.question_tex.is_some() {
                                self.question_tex.as_ref().unwrap()
                            } else {
                                self.brick_tex.as_ref().unwrap()
                            };
                            draw_texture_ex(
                                tex,
                                rect_x, rect_y,
                                WHITE, // Use WHITE to disable color tinting
                                DrawTextureParams {
                                    dest_size: Some(vec2(self.cell_size, self.cell_size)),
                                    ..Default::default()
                                }
                            );
                        } else {
                            draw_rectangle(rect_x + 1.0, rect_y + 1.0, self.cell_size - 2.0, self.cell_size - 2.0, colors[color_id]);
                        }
                    }
                }
            }
        }

        let board_w = engine.width as f32 * self.cell_size;
        let board_h = engine.height as f32 * self.cell_size;
        draw_rectangle_lines(offset_x, offset_y, board_w, board_h, 2.0, WHITE);

        // Sidebar Info
        let panel_x = offset_x + board_w + 30.0;
        draw_text(&format!("Score: {}", engine.score), panel_x, 50.0, 25.0, WHITE);
        draw_text(&format!("Lines: {}", engine.lines), panel_x, 80.0, 25.0, WHITE);
        draw_text(&format!("Level: {}", engine.level), panel_x, 110.0, 25.0, WHITE);
        
        let status = if engine.paused { "Status: PAUSED" } else { "Status: PLAYING" };
        let status_col = if engine.paused { RED } else { WHITE };
        draw_text(status, panel_x, 150.0, 25.0, status_col);
        
        let ai_str = match self.ai_mode { 0 => "OFF", 1 => "FAST", 2 => "SLOW", _ => "MANUAL_AI" };
        let ai_col = if self.ai_mode != 0 { GREEN } else { WHITE };
        draw_text(&format!("AI Mode: {}", ai_str), panel_x, 180.0, 25.0, ai_col);

        draw_text("NEXT PIECE:", panel_x, 220.0, 25.0, WHITE);
        for r in 0..4 {
            for c in 0..4 {
                if engine.next_piece.shape[r][c] != 0 {
                    let rect_x = panel_x + c as f32 * self.cell_size;
                    let rect_y = 250.0 + r as f32 * self.cell_size;
                    draw_rectangle(rect_x, rect_y, self.cell_size - 2.0, self.cell_size - 2.0, colors[engine.next_piece.shape_id as usize]);
                }
            }
        }

        // Diagnostics Panel
        if self.ai_mode == 3 {
            let diag_y = 390.0;
            draw_text("--- AI Diagnostics ---", panel_x, diag_y, 20.0, Color::new(1.0, 0.8, 0.2, 1.0));
            
            if let Some(eval) = &self.current_eval {
                let mut diag_texts = vec![
                    format!("Height (-0.51): {}", eval.agg_height),
                    format!("Lines (+0.76): {}", eval.cleared_lines),
                    format!("Holes (-0.36): {}", eval.holes),
                    format!("Bumpiness (-0.18): {}", eval.bumpiness),
                    format!("Curr Score: {:.2}", eval.score),
                ];
                
                if let Some(target) = &self.ai_target_move {
                    diag_texts.push(format!("AI Best (1-Ply): {:.2}", target.ply1_score));
                    diag_texts.push(format!("AI Best (2-Ply): {:.2}", target.score));
                }

                for (i, text) in diag_texts.iter().enumerate() {
                    let mut color = Color::new(0.8, 0.8, 0.8, 1.0);
                    if text.starts_with("Curr Score") { color = Color::new(0.4, 1.0, 1.0, 1.0); }
                    else if text.starts_with("AI Best") { color = Color::new(1.0, 0.4, 0.4, 1.0); }
                    
                    draw_text(text, panel_x, diag_y + 30.0 + i as f32 * 25.0, 18.0, color);
                }
            }
        }
    }
}
