#[derive(Clone, Debug)]
pub struct Piece {
    pub x: i32,
    pub y: i32,
    pub shape_id: u8,
    pub shape: [[u8; 4]; 4],
}

pub const SHAPES: [[[u8; 4]; 4]; 7] = [
    // 1: I (4x4 旋轉)
    [[0,0,0,0],[1,1,1,1],[0,0,0,0],[0,0,0,0]],
    // 2: J (3x3 旋轉，放置在左上角 3x3 區塊)
    [[1,0,0,0],[1,1,1,0],[0,0,0,0],[0,0,0,0]],
    // 3: L
    [[0,0,1,0],[1,1,1,0],[0,0,0,0],[0,0,0,0]],
    // 4: O (不旋轉)
    [[0,1,1,0],[0,1,1,0],[0,0,0,0],[0,0,0,0]],
    // 5: S
    [[0,1,1,0],[1,1,0,0],[0,0,0,0],[0,0,0,0]],
    // 6: T
    [[0,1,0,0],[1,1,1,0],[0,0,0,0],[0,0,0,0]],
    // 7: Z
    [[1,1,0,0],[0,1,1,0],[0,0,0,0],[0,0,0,0]],
];

impl Piece {
    pub fn new(x: i32, y: i32, id: u8) -> Self {
        Self { x, y, shape_id: id + 1, shape: SHAPES[id as usize] }
    }
    
    pub fn rotate(&mut self) {
        if self.shape_id == 4 { return; } // O 形不旋轉
        self.shape = self.get_rotated_shape();
    }
    
    pub fn get_rotated_shape(&self) -> [[u8; 4]; 4] {
        if self.shape_id == 4 { return self.shape; } // O 形
        
        let mut new_shape = [[0; 4]; 4];
        
        if self.shape_id == 1 {
            // I 形方塊使用 4x4 矩陣旋轉
            for r in 0..4 {
                for c in 0..4 {
                    new_shape[c][3 - r] = self.shape[r][c];
                }
            }
        } else {
            // J, L, S, T, Z 使用 3x3 矩陣旋轉 (因為這 5 種方塊的邏輯中心在 3x3 的正中間)
            // 將原本 3x3 區域的方塊轉置並翻轉
            for r in 0..3 {
                for c in 0..3 {
                    new_shape[c][2 - r] = self.shape[r][c];
                }
            }
        }
        new_shape
    }
}
