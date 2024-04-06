use anyhow::{anyhow, bail, Ok, Result};
use bitvec::vec::BitVec;
use rand::{prelude::*, seq::index::sample};
use std::{
    fmt::{self, Debug},
    usize,
};

use serde::{Deserialize, Serialize};

use crate::geometry::{Area, Point, Size};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
enum Cell {
    Bone,
    Empty(u8),
}

impl Cell {
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

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
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
    state: BitVec,
}

impl DigSite {
    fn symbol_at(&self, index: usize) -> Option<String> {
        let visibility = *self.state.get(index)?;

        if !visibility {
            Some("#".to_string())
        } else {
            let point = Area::from(self.dimensions).point_from_pos(index);
            let cell = self.get(point)?;
            Some(format!("{}", cell))
        }
    }

    fn size(&self) -> usize {
        self.dimensions.count()
    }

    /// Initialize all the cells in the board with empty states and positions
    fn build_board(count: usize) -> Board {
        vec![Cell::Empty(0); count]
    }

    fn build_state(count: usize) -> BitVec {
        BitVec::from_vec(vec![0; count])
    }

    pub fn new(size: Size) -> Self {
        let count = size.count();

        let board = DigSite::build_board(count);
        let state = DigSite::build_state(count);

        DigSite {
            dimensions: size,
            board,
            state,
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
        ds.state = DigSite::build_state(ds.dimensions.count());

        ds.clear_cell_state()
            .generate_bones(rng, bones, initial_pos)?
            .apply_cell_state()?;

        ds.flood_fill_visibility(initial_pos)?;

        Ok(ds)
    }

    fn in_bounds(&self, p: Point) -> bool {
        Area::from(self.dimensions).contains(p)
    }

    fn pos_from_point(&self, p: Point) -> usize {
        p.y * self.dimensions.x + p.x
    }

    fn get(&self, p: Point) -> Option<Cell> {
        self.board.get(self.pos_from_point(p)).copied()
    }

    fn set(&mut self, p: Point, c: Cell) -> Result<()> {
        if !self.in_bounds(p) {
            bail!("tried to set cell out of range")
        }
        let index = self.pos_from_point(p);
        self.board[index] = c;
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

            let cell = self.get(point).ok_or(anyhow!("placed bone out of range"))?;

            self.set(
                point,
                if matches!(cell, Cell::Empty(_)) {
                    placed_bones += 1;
                    Cell::Bone
                } else {
                    cell
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

                match self
                    .get(board_point)
                    .ok_or(anyhow!("accessing area around bone inaccessable"))?
                {
                    Cell::Bone => continue,
                    Cell::Empty(v) => self.set(board_point, Cell::Empty(v + 1))?,
                }
            }
        }

        Ok(())
    }

    fn flood_fill_visibility(&mut self, p: Point) -> Result<()> {
        let index = self.pos_from_point(p);

        let cell = self
            .get(p)
            .ok_or(anyhow!("Board is not synced with expected state size"))?;

        if index >= self.state.len() {
            bail!("State is not synced with expected board size");
        }

        if self.state[index] {
            return Ok(());
        }

        self.state.set(index, true);

        if matches!(cell, Cell::Empty(0)) {
            let dim_area = Area::from(self.dimensions);
            let flood_area = dim_area.intersecting_area(Area::around_point(p, 1));

            let cell_count = Size::from(flood_area).count();
            let area_normalized = flood_area.normalize();
            let area_offset = flood_area.0;

            for pos in 0..cell_count {
                let local_point = area_normalized.point_from_pos(pos);
                let board_point = local_point + area_offset;
                self.flood_fill_visibility(board_point)?;
            }

            Ok(())
        } else {
            // just leave alone
            Ok(())
        }
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
                let is_empty = matches!(cell, Cell::Empty(_));
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
                Cell::Bone,
            )?;
        }

        Ok(self)
    }

    /// Any of the scored cells on the board will get their warning score reset to 0
    fn clear_cell_state(&mut self) -> &mut Self {
        self.board.iter_mut().for_each(|c| {
            if matches!(c, Cell::Empty(_)) {
                *c = Cell::Empty(0)
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
            .enumerate()
            .filter_map(|(pos, c)| {
                if matches!(c, Cell::Bone) {
                    Some(pos)
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
                let target_cell = self
                    .get(board_point)
                    .ok_or(anyhow!("accessing area around bone oob"))?;

                if let Cell::Empty(v) = target_cell {
                    self.set(board_point, Cell::Empty(v + 1))?;
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
        self.board.iter().enumerate().for_each(|(i, _)| {
            let is_new_row = i % self.dimensions.x == 0;
            if is_new_row {
                print!("\n{} ", i / self.dimensions.x);
            }
            print!("{} ", self.symbol_at(i).unwrap_or("?".to_string()));
        })
    }
}
