use anyhow::{anyhow, bail, Ok, Result};
use bitvec::vec::BitVec;
use rand::{prelude::*, seq::index::sample};
use std::{
    collections::HashMap,
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
struct Player {
    symbol: char,
    pos: Point,
}
type Players = HashMap<char, Player>;

#[derive(Debug, Serialize, Deserialize)]
/// Digsite is a complete structure around the game board and state.
/// It contains the board, the dimensions, the initial position and the bones. Anything needed to
/// know during a run for a player.
pub struct DigSite {
    dimensions: Size,
    board: Board,
    state: BitVec,

    players: Players,
    spawn_pos: Option<Point>,
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
        let players = HashMap::new();

        DigSite {
            dimensions: size,
            board,
            state,
            players,
            spawn_pos: None,
        }
    }

    pub fn generate<R: Rng>(
        rng: &mut R,
        size: Size,
        bones: usize,
        initial_pos: Point,
        players: Option<Vec<char>>,
    ) -> Result<Self> {
        let mut ds = DigSite::new(size);

        ds.spawn_pos = Some(initial_pos);

        ds.board = DigSite::build_board(ds.dimensions.count());
        ds.state = DigSite::build_state(ds.dimensions.count());

        ds.clear_cell_state()
            .generate_bones(rng, bones, initial_pos)?
            .apply_cell_state()?;

        ds.flood_fill_visibility(initial_pos)?;

        if let Some(players) = players {
            for player in players {
                ds.add_player(player)?;
            }
        }

        Ok(ds)
    }

    fn add_player(&mut self, symbol: char) -> Result<()> {
        // TODO: Change this to adapt for upcoming changed player schema
        self.players.entry(symbol).or_insert(Player {
            symbol,
            pos: self.spawn_pos.ok_or(anyhow!(
                "no spawn point provided. was the board generated correctly?"
            ))?,
        });

        Ok(())
    }

    pub fn move_player(&mut self, symbol: char, p: Point) {
        if self.in_bounds(p) {
            self.players
                .entry(symbol)
                .and_modify(|player| player.pos = p);
        }
    }

    fn in_bounds(&self, p: Point) -> bool {
        Area::from(self.dimensions).contains(p)
    }

    fn pos_from_point(&self, p: Point) -> usize {
        p.y as usize * self.dimensions.x + p.x as usize
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

    fn output(&self) -> Vec<Vec<String>> {
        // Layout board
        let mut cells: Vec<_> = (0..self.size())
            .map(|i| self.symbol_at(i).unwrap_or(String::from("?")))
            .collect();

        // Place players
        for player in self.players.values() {
            let pos = self.pos_from_point(player.pos);
            cells[pos] = player.symbol.to_string();
        }

        // Pack everything into 2d vec
        cells
            .chunks(self.dimensions.x)
            .map(|r| Vec::from(r))
            .collect()
    }

    pub fn print(&self) {
        let data = self.output();

        let max_col_w = self.dimensions.x.saturating_sub(1).to_string().len();
        let max_row_w = self.dimensions.y.saturating_sub(1).to_string().len();

        let header = format!(
            "{:max_row_w$} {}",
            "",
            (0..self.dimensions.x).fold(String::new(), |acc, n| format!(
                "{}{:max_col_w$} ",
                acc,
                n,
                max_col_w = max_col_w
            )),
            max_row_w = max_row_w
        );

        println!("{}", header);

        for (y, row) in data.iter().enumerate() {
            println!(
                "{:max_row_w$}{}",
                y,
                row.iter().fold(String::new(), |acc, symbol| {
                    format!("{} {:max_col_w$}", acc, symbol, max_col_w = max_col_w)
                }),
                max_row_w = max_row_w
            )
        }

        for player in self.players.values() {
            println!("{}: {}", player.symbol, player.pos);
        }
    }
}
