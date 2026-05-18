use crate::core::simulator::Simulator;
use crate::core::board::TetrisEngine;
use crate::core::pieces::Piece;
use rayon::prelude::*;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Move {
    pub rotation: u8,
    pub x: i32,
    pub drop_y: i32,
    pub shape: [[u8; 4]; 4],
    pub score: f32,
    pub ply1_score: f32,
    pub sim_grid: Vec<Vec<u8>>,
    pub cleared_lines: u32,
}

pub struct Evaluator;

impl Evaluator {
    pub fn evaluate(grid: &Vec<Vec<u8>>, cleared_lines: u32) -> f32 {
        let w_height = -0.51;
        let w_lines = 0.76;
        let w_holes = -0.36;
        let w_bumpiness = -0.18;

        let width = grid[0].len();
        let height = grid.len();
        let mut heights = vec![0; width];

        for x in 0..width {
            for y in 0..height {
                if grid[y][x] != 0 {
                    heights[x] = (height - y) as i32;
                    break;
                }
            }
        }

        let agg_height: i32 = heights.iter().sum();
        
        let mut holes = 0;
        for x in 0..width {
            let top_y = height as i32 - heights[x];
            for y in top_y..height as i32 {
                if grid[y as usize][x] == 0 { holes += 1; }
            }
        }

        let mut bumpiness = 0;
        for i in 0..width - 1 {
            bumpiness += (heights[i] - heights[i+1]).abs();
        }

        (w_height * agg_height as f32) + 
        (w_lines * cleared_lines as f32) + 
        (w_holes * holes as f32) + 
        (w_bumpiness * bumpiness as f32)
    }
}

pub struct MoveGenerator;

impl MoveGenerator {
    pub fn get_possible_moves(piece: &Piece, width: usize, height: usize, base_grid: &Vec<Vec<u8>>) -> Vec<Move> {
        let mut possible_moves = Vec::new();
        let mut test_piece = piece.clone();
        let mut seen_shapes = Vec::new();

        for rot in 0..4 {
            if !seen_shapes.contains(&test_piece.shape) {
                seen_shapes.push(test_piece.shape);

                let mut min_c = 4;
                let mut max_c = -1;
                for r in 0..4 {
                    for c in 0..4 {
                        if test_piece.shape[r][c] != 0 {
                            if (c as i32) < min_c { min_c = c as i32; }
                            if (c as i32) > max_c { max_c = c as i32; }
                        }
                    }
                }
                
                if min_c <= max_c {
                    let start_x = -min_c;
                    let end_x = width as i32 - max_c;

                    for x in start_x..end_x {
                        let mut sim = Simulator::new(width, height, base_grid);
                        if sim.is_valid_position(&test_piece, x, 0) {
                            let drop_y = sim.drop_piece(&test_piece, x, 0);
                            sim.lock_piece(&test_piece, x, drop_y);
                            let cleared = sim.clear_lines();
                            let score = Evaluator::evaluate(&sim.grid, cleared);

                            possible_moves.push(Move {
                                rotation: rot,
                                x,
                                drop_y,
                                shape: test_piece.shape,
                                score: std::f32::NEG_INFINITY,
                                ply1_score: score,
                                sim_grid: sim.grid.clone(),
                                cleared_lines: cleared,
                            });
                        }
                    }
                }
            }
            test_piece.rotate();
        }
        possible_moves
    }
}

pub struct AIBot;

impl AIBot {
    pub fn get_best_move(engine: &TetrisEngine) -> Option<Move> {
        let current_piece = &engine.current_piece;
        let next_piece = &engine.next_piece;
        let width = engine.width;
        let height = engine.height;
        let base_grid = &engine.grid;

        let mut moves_ply1 = MoveGenerator::get_possible_moves(current_piece, width, height, base_grid);

        if moves_ply1.is_empty() {
            return None;
        }

        // Ply 2: Parallel evaluation
        moves_ply1.par_iter_mut().for_each(|m1| {
            let moves_ply2 = MoveGenerator::get_possible_moves(next_piece, width, height, &m1.sim_grid);
            
            if moves_ply2.is_empty() {
                m1.score = std::f32::NEG_INFINITY;
            } else {
                let mut best_score_ply2 = std::f32::NEG_INFINITY;
                for m2 in moves_ply2 {
                    let eval_score = Evaluator::evaluate(&m2.sim_grid, m1.cleared_lines + m2.cleared_lines);
                    if eval_score > best_score_ply2 {
                        best_score_ply2 = eval_score;
                    }
                }
                m1.score = best_score_ply2;
            }
        });

        let mut best_move: Option<Move> = None;
        let mut best_score = std::f32::NEG_INFINITY;

        for m1 in &moves_ply1 {
            if m1.score > best_score {
                best_score = m1.score;
                best_move = Some(m1.clone());
            }
        }

        // Fallback: If all 2-ply evaluations lead to game over (-inf), fallback to best 1-ply move
        if best_move.is_none() || best_score == std::f32::NEG_INFINITY {
            let mut best_score_fallback = std::f32::NEG_INFINITY;
            for m1 in &moves_ply1 {
                if m1.ply1_score > best_score_fallback {
                    best_score_fallback = m1.ply1_score;
                    best_move = Some(m1.clone());
                }
            }
        }

        best_move
    }
}
