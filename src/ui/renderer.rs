use macroquad::prelude::*;
use crate::core::board::TetrisEngine;

pub enum GameState {
    Menu,
    Playing,
    GameOver,
}

pub struct Renderer {
    state: GameState,
    config_width: usize,
    config_height: usize,
    menu_selection: u8,
    engine: Option<TetrisEngine>,
    fall_time: f32,
    fall_speed: f32,
    cell_size: f32,
    key_repeat_timer: f32,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            state: GameState::Menu,
            config_width: 10,
            config_height: 20,
            menu_selection: 0,
            engine: None,
            fall_time: 0.0,
            fall_speed: 0.5,
            cell_size: 30.0,
            key_repeat_timer: 0.0,
        }
    }

    pub async fn run(&mut self) {
        loop {
            clear_background(Color::new(0.12, 0.12, 0.12, 1.0));

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
            self.menu_selection = (self.menu_selection + 2) % 3;
        } else if is_key_pressed(KeyCode::Down) {
            self.menu_selection = (self.menu_selection + 1) % 3;
        }

        if is_key_pressed(KeyCode::Left) {
            if self.menu_selection == 0 && self.config_width > 5 { self.config_width -= 1; }
            if self.menu_selection == 1 && self.config_height > 10 { self.config_height -= 1; }
        } else if is_key_pressed(KeyCode::Right) {
            if self.menu_selection == 0 && self.config_width < 30 { self.config_width += 1; }
            if self.menu_selection == 1 && self.config_height < 40 { self.config_height += 1; }
        } else if is_key_pressed(KeyCode::Enter) {
            if self.menu_selection == 2 {
                self.start_game();
            } else {
                self.menu_selection = (self.menu_selection + 1) % 3;
            }
        }

        let title = "LUCY TETRIS (RUST)";
        draw_text(title, screen_width()/2.0 - 150.0, 100.0, 40.0, WHITE);

        let items = [
            format!("Board Width: < {} >", self.config_width),
            format!("Board Height: < {} >", self.config_height),
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

        let max_board_w = screen_width() - 250.0;
        let max_board_h = screen_height() - 40.0;
        let cell_w = max_board_w / self.config_width as f32;
        let cell_h = max_board_h / self.config_height as f32;
        self.cell_size = 30.0_f32.min(cell_w).min(cell_h);
    }

    fn update_playing(&mut self) {
        let engine = self.engine.as_mut().unwrap();
        
        if is_key_pressed(KeyCode::Up) {
            engine.toggle_pause();
        }

        if !engine.paused && !engine.game_over {
            let dt = get_frame_time();
            self.fall_time += dt;

            let current_speed = (self.fall_speed - (engine.level as f32 - 1.0) * 0.05).max(0.1);
            
            if self.fall_time >= current_speed {
                engine.move_piece(0, 1);
                self.fall_time = 0.0;
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

            if dx != 0 || dy != 0 {
                engine.move_piece(dx, dy);
            }
        }

        self.draw_game();

        if engine.game_over {
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

        for y in 0..engine.height {
            for x in 0..engine.width {
                let rect_x = offset_x + x as f32 * self.cell_size;
                let rect_y = offset_y + y as f32 * self.cell_size;
                draw_rectangle_lines(rect_x, rect_y, self.cell_size, self.cell_size, 1.0, DARKGRAY);
                
                let cell = engine.grid[y][x];
                if cell != 0 {
                    draw_rectangle(rect_x + 1.0, rect_y + 1.0, self.cell_size - 2.0, self.cell_size - 2.0, colors[cell as usize]);
                }
            }
        }

        for r in 0..4 {
            for c in 0..4 {
                if engine.current_piece.shape[r][c] != 0 {
                    let y = engine.current_piece.y + r as i32;
                    let x = engine.current_piece.x + c as i32;
                    if y >= 0 {
                        let rect_x = offset_x + x as f32 * self.cell_size;
                        let rect_y = offset_y + y as f32 * self.cell_size;
                        draw_rectangle(rect_x + 1.0, rect_y + 1.0, self.cell_size - 2.0, self.cell_size - 2.0, colors[engine.current_piece.shape_id as usize]);
                    }
                }
            }
        }

        let board_w = engine.width as f32 * self.cell_size;
        let board_h = engine.height as f32 * self.cell_size;
        draw_rectangle_lines(offset_x, offset_y, board_w, board_h, 2.0, WHITE);

        let panel_x = offset_x + board_w + 30.0;
        draw_text(&format!("Score: {}", engine.score), panel_x, 50.0, 25.0, WHITE);
        draw_text(&format!("Lines: {}", engine.lines), panel_x, 80.0, 25.0, WHITE);
        draw_text(&format!("Level: {}", engine.level), panel_x, 110.0, 25.0, WHITE);
        
        let status = if engine.paused { "Status: PAUSED" } else { "Status: PLAYING" };
        let status_col = if engine.paused { RED } else { WHITE };
        draw_text(status, panel_x, 150.0, 25.0, status_col);

        draw_text("NEXT PIECE:", panel_x, 200.0, 25.0, WHITE);
        for r in 0..4 {
            for c in 0..4 {
                if engine.next_piece.shape[r][c] != 0 {
                    let rect_x = panel_x + c as f32 * self.cell_size;
                    let rect_y = 230.0 + r as f32 * self.cell_size;
                    draw_rectangle(rect_x, rect_y, self.cell_size - 2.0, self.cell_size - 2.0, colors[engine.next_piece.shape_id as usize]);
                }
            }
        }
    }
}
