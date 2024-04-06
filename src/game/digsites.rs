use anyhow::{anyhow, bail, Ok, Result};
use rand::prelude::*;
use std::{
    fmt::{self},
    usize,
};

static SPACING: usize = 2;

use serde::{Deserialize, Serialize};

use crate::geometry::{Area, Point, Size};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
enum CellType {
    Bone,
    Empty(u8),
}

impl CellType {
    fn symbol(&self) -> String {
        match self {
            Self::Empty(v) => match v {
                0 => ".".to_string(),
                _ => format!("{}", v),
            },
            Self::Bone => "b".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct Cell {
    pos: usize,
    t: CellType,
}

impl Cell {
    fn new(t: CellType, pos: usize) -> Self {
        Cell { t, pos }
    }

    fn set_pos(mut self, pos: usize) -> Self {
        self.pos = pos;
        self
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "|{:>width$} ", self.t.symbol(), width = SPACING)
    }
}

type Board = Vec<Cell>;

trait BoardMethods {}

impl BoardMethods for Board {}

#[derive(Debug, Serialize, Deserialize)]
/// Digsite is a complete structure around the game board and state.
/// It contains the board, the dimensions, the initial position and the bones. Anything needed to
/// know during a run for a player.
pub struct DigSite {
    dimensions: Size,
    initial_position: Point,
    board: Board,
    bones: usize,
}

impl DigSite {
    fn size(&self) -> usize {
        self.dimensions.count()
    }

    /// Initialize all the cells in the board with empty states and positions
    fn build_board(size: usize) -> Board {
        (0..size)
            .map(|i| Cell {
                t: CellType::Empty(0),
                pos: i,
            })
            .collect()
    }

    pub fn new(x: usize, y: usize, bones: usize, init_x: usize, init_y: usize) -> Self {
        let dimensions = Size { x, y };
        let initial_position = Point {
            x: init_x,
            y: init_y,
        };

        let board = DigSite::build_board(x * y);

        DigSite {
            dimensions,
            initial_position,
            board,
            bones,
        }
    }

    fn in_bounds(&self, p: Point) -> bool {
        Area::from(self.dimensions).contains(p)
    }

    fn get(&self, p: Point) -> Result<Cell> {
        if !self.in_bounds(p) {
            bail!("tried to get cell out of range")
        }

        self.board
            .get(p.y * self.dimensions.x + p.x)
            .copied()
            .ok_or_else(|| anyhow!("out of range"))
    }

    fn set(&mut self, p: Point, c: Cell) -> Result<()> {
        if !self.in_bounds(p) {
            bail!("tried to set cell out of range")
        }
        let index = p.y * self.dimensions.x + p.x; // Set the position of the cell
        self.board[index] = c.set_pos(index);
        Ok(())
    }

    /// Should only be called during initialization
    pub fn assign_bones<R: Rng>(&mut self, mut rng: R) -> Result<()> {
        let dimension_area = Area::from(self.dimensions);
        self.board = DigSite::build_board(self.size());

        let mut placed_bones: usize = 0;

        let invalid_positions = Area::around_point(self.initial_position, 1);

        while placed_bones < self.bones {
            let position = rng.gen_range(0..self.size());
            let point = dimension_area.point_from_pos(position);

            if invalid_positions.contains(point) {
                continue;
            }

            let cell = self.get(point)?;

            self.set(
                point,
                match cell.t {
                    CellType::Bone => cell,
                    CellType::Empty(_) => {
                        placed_bones += 1;
                        Cell::new(CellType::Bone, position)
                    }
                },
            )?;

            let bone_radius = Area::around_point(point, 1);
            let bone_cell_area = dimension_area.intersecting_area(bone_radius);

            let bone_cell_offset = bone_cell_area.0;

            let cell_count = Size::from(bone_cell_area).count();

            let bca_normal = bone_cell_area.normalize();

            for pos in 0..cell_count {
                let local_point = bca_normal.point_from_pos(pos);
                let board_point = local_point + bone_cell_offset;

                match self.get(board_point)?.t {
                    CellType::Bone => continue,
                    CellType::Empty(v) => {
                        self.set(board_point, Cell::new(CellType::Empty(v + 1), 0))?
                    }
                }
            }
        }

        Ok(())
    }

    pub fn print(&self) {
        let mut first:bool = false;

        let mut line = String::from("");
        vec![0; self.dimensions.x*(SPACING+2)]
        .iter()
        .enumerate()
        .for_each(|(_, _)| {
            line.push('-');
        });

        println!("{}", self.dimensions);
        for _ in 0..=SPACING {
            print!(" ");
        }
        vec![0; self.dimensions.x]
            .iter()
            .enumerate()
            .for_each(|(i, _)| {
                print!("{:>width$}  ", i, width = SPACING);
                // print!("|");
            });
        self.board.iter().enumerate().for_each(|(i, cell)| {
            let is_new_row = i % self.dimensions.x == 0;
            if is_new_row {
                if !first {
                    print!("\n   {}", line);
                    first = true;
                } else {
                    print!("|\n   {}", line);
                }
                print!("\n{:<width$}", i / self.dimensions.x, width = SPACING);
            }
            print!("{}", cell);
        });
        print!("|\n   {}", line);
    }
}
