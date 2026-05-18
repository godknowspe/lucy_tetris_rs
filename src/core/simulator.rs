use crate::core::pieces::Piece;

pub struct Simulator {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<u8>>,
}

impl Simulator {
    pub fn new(width: usize, height: usize, grid: &Vec<Vec<u8>>) -> Self {
        Self { width, height, grid: grid.clone() }
    }

    pub fn is_valid_position(&self, piece: &Piece, x: i32, y: i32) -> bool {
        for r in 0..4 {
            for c in 0..4 {
                if piece.shape[r][c] != 0 {
                    let px = x + c as i32;
                    let py = y + r as i32;
                    if px < 0 || px >= self.width as i32 || py >= self.height as i32 { return false; }
                    if py >= 0 && self.grid[py as usize][px as usize] != 0 { return false; }
                }
            }
        }
        true
    }

    pub fn drop_piece(&self, piece: &Piece, x: i32, y: i32) -> i32 {
        let mut drop_y = y;
        while self.is_valid_position(piece, x, drop_y + 1) { drop_y += 1; }
        drop_y
    }

    pub fn lock_piece(&mut self, piece: &Piece, x: i32, y: i32) {
        for r in 0..4 {
            for c in 0..4 {
                if piece.shape[r][c] != 0 {
                    let py = y + r as i32;
                    let px = x + c as i32;
                    if py >= 0 && py < self.height as i32 {
                        // 使用方塊 ID，或是用 99 代表 Simulated
                        self.grid[py as usize][px as usize] = piece.shape_id;
                    }
                }
            }
        }
    }

    pub fn clear_lines(&mut self) -> u32 {
        let mut new_grid = Vec::new();
        let mut cleared = 0;
        for row in &self.grid {
            if row.iter().all(|&cell| cell != 0) {
                cleared += 1;
            } else {
                new_grid.push(row.clone());
            }
        }
        for _ in 0..cleared { new_grid.insert(0, vec![0; self.width]); }
        self.grid = new_grid;
        cleared
    }
}
