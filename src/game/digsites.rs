use anyhow::{anyhow, bail, Ok, Result};
use rand::{prelude::*, seq::index::sample};
use std::{fmt, usize};

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

    fn set_type(mut self, t: CellType) -> Self {
        self.t = t;
        self
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.t.symbol())
    }
}

type Board = Vec<Cell>;

#[derive(Debug, Serialize, Deserialize)]
/// Digsite is a complete structure around the game board and state.
/// It contains the board, the dimensions, the initial position and the bones. Anything needed to
/// know during a run for a player.
pub struct DigSite {
    dimensions: Size,
    board: Board,
}

impl DigSite {
    fn size(&self) -> usize {
        self.dimensions.count()
    }

    /// Initialize all the cells in the board with empty states and positions
    fn build_board(cell_count: usize) -> Board {
        (0..cell_count)
            .map(|i| Cell {
                t: CellType::Empty(0),
                pos: i,
            })
            .collect()
    }

    pub fn new(size: Size) -> Self {
        let board = DigSite::build_board(size.count());
        DigSite {
            dimensions: size,
            board,
        }
    }

    pub fn generate<R: Rng>(
        rng: &mut R,
        size: Size,
        bones: usize,
        initial_pos: Point,
    ) -> Result<Self> {
        let mut ds = DigSite::new(size);

        ds.board = DigSite::build_board(ds.dimensions.count());

        ds.clear_cell_state()
            .generate_bones(rng, bones, initial_pos)?
            .apply_cell_state()?;

        Ok(ds)
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
    pub fn assign_bones<R: Rng>(
        &mut self,
        mut rng: R,
        bones: usize,
        initial_pos: Point,
    ) -> Result<()> {
        let dimension_area = Area::from(self.dimensions);

        let mut placed_bones: usize = 0;

        let invalid_positions = Area::around_point(initial_pos, 1);

        while placed_bones < bones {
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

            let cell_count = Size::from(bone_cell_area).count();
            let bca_normal = bone_cell_area.normalize();
            let bone_cell_offset = bone_cell_area.0;

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

    /// Distributes a specified number of bones around the map, avoiding the immediate area around the initial position.
    /// This does not alter cells that are not empty or remove any existing state.
    fn generate_bones<R: Rng>(
        &mut self,
        rng: &mut R,
        num_bones: usize,
        initial_pos: Point,
    ) -> Result<&mut Self> {
        let dim_area = Area::from(self.dimensions);
        let exclusion_zone = dim_area.intersecting_area(Area::around_point(initial_pos, 1));

        // Identify all positions on the board that are empty and not in the exclusion zone.
        let potential_locations: Vec<_> = self
            .board
            .iter()
            .enumerate()
            .filter_map(|(pos, cell)| {
                let is_empty = matches!(cell.t, CellType::Empty(_));
                let point = dim_area.point_from_pos(pos);
                let is_excluded = exclusion_zone.contains(point);
                if is_empty && !is_excluded {
                    Some(point)
                } else {
                    None
                }
            })
            .collect();

        // Randomly select positions to place bones, ensuring no duplication.
        let selected_positions = sample(rng, potential_locations.len(), num_bones);

        for idx in selected_positions {
            self.set(
                *potential_locations
                    .get(idx)
                    .ok_or(anyhow!("invalid sample"))?,
                Cell {
                    t: CellType::Bone,
                    pos: 0,
                },
            )?;
        }

        Ok(self)
    }
    /// Any of the scored cells on the board will get their warning score reset to 0
    fn clear_cell_state(&mut self) -> &mut Self {
        self.board.iter_mut().for_each(|c| {
            if let CellType::Empty(_) = c.t {
                c.t = CellType::Empty(0)
            }
        });

        self
    }

    /// Set the funny minesweeper numbers around each bone
    fn apply_cell_state(&mut self) -> Result<&mut Self> {
        let dim_area = Area::from(self.dimensions);

        // Clone bones for the positions
        let bones: Vec<_> = self
            .board
            .iter()
            .filter_map(|c| {
                if matches!(c.t, CellType::Bone) {
                    Some(c.pos)
                } else {
                    None
                }
            })
            .collect();

        // For each bone update the neighbors
        for pos in bones {
            let point = dim_area.point_from_pos(pos);
            let bone_radius = Area::around_point(point, 1);
            let bone_cell_area = dim_area.intersecting_area(bone_radius);

            let cell_count = Size::from(bone_cell_area).count();
            let bca_normal = bone_cell_area.normalize();
            let bone_cell_offset = bone_cell_area.0;

            for pos in 0..cell_count {
                let local_point = bca_normal.point_from_pos(pos);
                let board_point = local_point + bone_cell_offset;
                let target_cell = self.get(board_point)?;

                if let CellType::Empty(v) = target_cell.t {
                    self.set(board_point, target_cell.set_type(CellType::Empty(v + 1)))?;
                }
            }
        }

        Ok(self)
    }

    pub fn print(&self) {
        println!("{}", self.dimensions);
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
