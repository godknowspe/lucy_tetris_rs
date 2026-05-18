#[derive(Clone, Debug)]
pub struct Piece {
    pub x: i32,
    pub y: i32,
    pub shape_id: u8,
    pub shape: [[u8; 4]; 4],
}

pub const SHAPES: [[[u8; 4]; 4]; 7] = [
    // 1: I
    [[0,0,0,0],[1,1,1,1],[0,0,0,0],[0,0,0,0]],
    // 2: J
    [[1,0,0,0],[1,1,1,0],[0,0,0,0],[0,0,0,0]],
    // 3: L
    [[0,0,1,0],[1,1,1,0],[0,0,0,0],[0,0,0,0]],
    // 4: O
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
        self.shape = self.get_rotated_shape();
    }
    
    pub fn get_rotated_shape(&self) -> [[u8; 4]; 4] {
        let mut new_shape = [[0; 4]; 4];
        for r in 0..4 {
            for c in 0..4 {
                new_shape[c][3 - r] = self.shape[r][c];
            }
        }
        new_shape
    }
}
