use crate::core::pieces::Piece;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub struct TetrisEngine {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<u8>>,
    pub current_piece: Piece,
    pub next_piece: Piece,
    pub score: u32,
    pub lines: u32,
    pub level: u32,
    pub game_over: bool,
    pub paused: bool,
    rng: StdRng,
}

impl TetrisEngine {
    pub fn new(width: usize, height: usize, seed: u64) -> Self {
        let rng = StdRng::seed_from_u64(seed);
        let mut engine = Self {
            width,
            height,
            grid: vec![vec![0; width]; height],
            current_piece: Piece::new(0, 0, 0),
            next_piece: Piece::new(0, 0, 0),
            score: 0,
            lines: 0,
            level: 1,
            game_over: false,
            paused: false,
            rng,
        };
        engine.current_piece = engine.spawn_piece();
        engine.next_piece = engine.spawn_piece();
        engine
    }

    fn spawn_piece(&mut self) -> Piece {
        let id = self.rng.gen_range(0..7) as u8;
        Piece::new((self.width / 2 - 2) as i32, 0, id)
    }

    pub fn is_valid_position(&self, piece: &Piece, adj_x: i32, adj_y: i32, rotated_shape: Option<[[u8;4];4]>) -> bool {
        let shape = rotated_shape.unwrap_or(piece.shape);
        for r in 0..4 {
            for c in 0..4 {
                if shape[r][c] != 0 {
                    let x = piece.x + c as i32 + adj_x;
                    let y = piece.y + r as i32 + adj_y;
                    if x < 0 || x >= self.width as i32 || y >= self.height as i32 {
                        return false;
                    }
                    if y >= 0 && self.grid[y as usize][x as usize] != 0 {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn move_piece(&mut self, dx: i32, dy: i32) -> bool {
        if self.game_over || self.paused { return false; }
        if self.is_valid_position(&self.current_piece, dx, dy, None) {
            self.current_piece.x += dx;
            self.current_piece.y += dy;
            true
        } else {
            if dy > 0 {
                self.lock_piece();
            }
            false
        }
    }

    pub fn rotate_piece(&mut self) {
        if self.game_over || self.paused { return; }
        let rotated = self.current_piece.get_rotated_shape();
        if self.is_valid_position(&self.current_piece, 0, 0, Some(rotated)) {
            self.current_piece.rotate();
        }
    }

    pub fn toggle_pause(&mut self) {
        if !self.game_over {
            self.paused = !self.paused;
        }
    }

    fn lock_piece(&mut self) {
        for r in 0..4 {
            for c in 0..4 {
                if self.current_piece.shape[r][c] != 0 {
                    let y = self.current_piece.y + r as i32;
                    let x = self.current_piece.x + c as i32;
                    if y >= 0 && y < self.height as i32 {
                        self.grid[y as usize][x as usize] = self.current_piece.shape_id;
                    }
                }
            }
        }
        self.clear_lines();
        self.current_piece = self.next_piece.clone();
        self.next_piece = self.spawn_piece();
        if !self.is_valid_position(&self.current_piece, 0, 0, None) {
            self.game_over = true;
        }
    }

    fn clear_lines(&mut self) {
        let mut new_grid = Vec::new();
        let mut cleared = 0;
        for row in &self.grid {
            if row.iter().all(|&cell| cell != 0) {
                cleared += 1;
            } else {
                new_grid.push(row.clone());
            }
        }
        for _ in 0..cleared {
            new_grid.insert(0, vec![0; self.width]);
        }
        self.grid = new_grid;
        self.lines += cleared;
        let points = [0, 100, 300, 500, 800];
        self.score += points[cleared as usize] * self.level;
        self.level = 1 + self.lines / 10;
    }
}
