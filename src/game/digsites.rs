use rand::prelude::*;
use std::{fmt, usize};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
pub struct Point {
    x: usize,
    y: usize,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl Point {
    pub fn size(&self) -> usize {
        self.x * self.y
    }

    /// Calculate a box around the point
    pub fn get_range(&self, steps: usize) -> (Point, Point) {
        (
            Point {
                x: self.x.saturating_sub(steps),
                y: self.y.saturating_sub(steps),
            },
            Point {
                x: self.x.saturating_add(steps),
                y: self.y.saturating_add(steps),
            },
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
pub enum CellType {
    Bone,
    Empty,
}

impl CellType {
    fn to_str(&self) -> &str {
        match self {
            Self::Empty => ".",
            Self::Bone => "b",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Cell {
    pos: Point,
    t: CellType,
}

impl Cell {
    pub fn new(t: CellType, pos: Point) -> Self {
        Cell { t, pos }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.t.to_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DigSite {
    dimensions: Point,
    initial_position: Point,
    board: Vec<Vec<Cell>>,
    bones: usize,
}

/// Initialize a board with empty cells with the correct position
fn build_board(x: usize, y: usize) -> Vec<Vec<Cell>> {
    vec![vec![Cell::new(CellType::Empty, Point { x, y }); x]; y]
        .iter_mut()
        .enumerate()
        .map(|(y, row)| {
            row.iter_mut()
                .enumerate()
                .map(|(x, cell)| Cell {
                    t: cell.t,
                    pos: Point { x, y },
                })
                .collect()
        })
        .collect()
}

impl DigSite {
    pub fn new(x: usize, y: usize, bones: usize, init_x: usize, init_y: usize) -> Self {
        let dimensions = Point { x, y };
        let initial_position = Point {
            x: init_x,
            y: init_y,
        };

        let board = build_board(x, y);

        DigSite {
            dimensions,
            initial_position,
            board,
            bones,
        }
    }

    fn get_point_from_pos(&self, pos: usize) -> Point {
        Point {
            x: pos % self.dimensions.y,
            y: pos / self.dimensions.x,
        }
    }

    fn get_cell_at_pos(&self, pos: usize) -> Cell {
        let position = self.get_point_from_pos(pos);
        self.board[position.y][position.x]
    }

    fn set_cell(&mut self, pos: usize, cell: Cell) {
        let position = self.get_point_from_pos(pos);
        self.board[position.y][position.x] = cell;
    }

    pub fn assign_bombs<R: Rng>(&mut self, mut rng: R) {
        let mut placed_bones: usize = 0;

        // let invalid_positions = self.initial_position.get_range(1);

        while placed_bones < self.bones {
            let position = rng.gen_range(0..self.dimensions.size());
            let cell = self.get_cell_at_pos(position);

            if cell.t == CellType::Bone {
                continue;
            }

            self.set_cell(
                position,
                Cell {
                    t: CellType::Bone,
                    pos: cell.pos,
                },
            );

            placed_bones += 1;
        }
    }

    pub fn print(&self) {
        print!("  ");
        vec![0; self.dimensions.x]
            .iter()
            .enumerate()
            .for_each(|(i, _)| {
                print!("{} ", i);
            });
        println!("");
        self.board.iter().enumerate().for_each(|(y, row)| {
            print!("{} ", y);
            row.iter().enumerate().for_each(|(_x, cell)| {
                print!("{} ", cell);
            });
            println!("");
        })
    }
}
