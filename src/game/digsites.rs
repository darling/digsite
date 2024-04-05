use anyhow::{anyhow, bail, Ok, Result};
use rand::prelude::*;
use std::{
    fmt::{self, Display},
    ops::Add,
    usize,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
/// A point in a 2D space
///
/// Note: Points are positive only
struct Point {
    x: usize,
    y: usize,
}

const EMPTY_POINT: Point = Point { x: 0, y: 0 };

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl Point {
    fn tuple(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    /// Calculate a box around the point
    ///
    /// This does not account for negative values, nor boundaries
    fn get_range(&self, steps: usize) -> Area {
        Area(
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

    pub fn to_area(&self) -> Area {
        Area(EMPTY_POINT, self.clone())
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, p: Point) -> Point {
        Point {
            x: self.x.saturating_add(p.x),
            y: self.y.saturating_add(p.y),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct Area(Point, Point);

trait AreaMethods {
    fn tuple(&self) -> (Point, Point);
    fn contains(&self, p: Point) -> bool;
    fn normalized(&self) -> Area;
    fn point_from_pos(&self, pos: usize) -> Point;
    fn intersecting_area(&self, a: Area) -> Area;
}

impl Display for Area {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl Area {
    fn size(&self) -> Size {
        let (x, y) = self.normalized().1.tuple();
        Size {
            x: x.saturating_add(1),
            y: y.saturating_add(1),
        }
    }

    fn tuple(&self) -> (Point, Point) {
        (self.0, self.1)
    }

    fn contains(&self, p: Point) -> bool {
        let (min, max) = self.tuple();
        p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y
    }

    fn normalized(&self) -> Area {
        let (min, max) = self.tuple();
        Area(
            EMPTY_POINT,
            Point {
                x: max.x.saturating_sub(min.x),
                y: max.y.saturating_sub(min.y),
            },
        )
    }

    fn point_from_pos(&self, pos: usize) -> Point {
        let n = self.size();
        Point {
            x: pos % n.x,
            y: pos / n.x,
        }
    }

    fn intersecting_area(&self, a: Area) -> Area {
        Area(
            Point {
                x: self.0.x.max(a.0.x),
                y: self.0.y.max(a.0.y),
            },
            Point {
                x: self.1.x.min(a.1.x),
                y: self.1.y.min(a.1.y),
            },
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
enum CellType {
    Bone,
    Empty(u8),
}

impl CellType {
    fn to_string(&self) -> String {
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
    pub fn new(t: CellType, pos: usize) -> Self {
        Cell { t, pos }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.t.to_string())
    }
}

type Board = Vec<Cell>;

trait BoardMethods {}

impl BoardMethods for Board {}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
/// For things like the dimension of the board, where a point would ruin conversion math.
struct Size {
    x: usize,
    y: usize,
}

impl Size {
    fn to_point(&self) -> Point {
        Point {
            x: self.x.saturating_sub(1),
            y: self.y.saturating_sub(1),
        }
    }

    fn count(&self) -> usize {
        self.x * self.y
    }

    fn to_area(&self) -> Area {
        self.to_point().to_area()
    }
}

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

    fn build_board(size: usize) -> Board {
        let mut b = vec![];
        for i in 0..size {
            b.push(Cell {
                t: CellType::Empty(0),
                pos: i,
            });
        }
        return b;
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

    fn get(&self, p: Point) -> Result<Cell> {
        if p.x >= self.dimensions.x {
            bail!("out of range")
        }
        if p.y >= self.dimensions.y {
            bail!("out of range")
        }

        self.board
            .get(p.y * self.dimensions.x + p.x)
            .copied()
            .ok_or_else(|| anyhow!("out of range"))
    }

    fn set(&mut self, p: Point, c: Cell) -> Result<()> {
        if p.x >= self.dimensions.x {
            bail!("out of range")
        }
        if p.y >= self.dimensions.y {
            bail!("out of range")
        }

        let index = p.y * self.dimensions.x + p.x;
        let cell = Cell::new(c.t, index);
        self.board[index] = cell;

        Ok(())
    }

    /// Should only be called during initialization
    pub fn assign_bones<R: Rng>(&mut self, mut rng: R) -> Result<()> {
        self.board = DigSite::build_board(self.size());

        let mut placed_bones: usize = 0;

        let invalid_positions = self.initial_position.get_range(1);

        while placed_bones < self.bones {
            let position = rng.gen_range(0..self.size());

            let point = self.dimensions.to_area().point_from_pos(position);

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

            let bone_radius = point.get_range(1);
            let bone_cell_area = self.dimensions.to_area().intersecting_area(bone_radius);

            let bone_cell_offset = bone_cell_area.0;

            let count = bone_cell_area.size().count();

            let bca_normal = bone_cell_area.normalized();

            for pos in 0..count {
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
        println!("{}", self.dimensions.to_area());
        print!("  ");
        vec![0; self.dimensions.x]
            .iter()
            .enumerate()
            .for_each(|(i, _)| {
                print!("{} ", i);
            });
        self.board.iter().enumerate().for_each(|(i, cell)| {
            let is_new_row = i % self.dimensions.x == 0;
            if is_new_row {
                print!("\n{} ", i / self.dimensions.x);
            }
            print!("{} ", cell);
        })
    }
}
