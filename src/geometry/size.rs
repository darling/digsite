use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::Area;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
/// For things like the dimension of the board, where a point would ruin conversion math.
/// This is different from an area in the sense that it's inclusive and one size larger than area.
pub struct Size {
    pub x: usize,
    pub y: usize,
}

impl Size {
    /// The total amount of units the size holds.
    ///
    /// if the size is (10,10) the count is 100
    pub fn count(&self) -> usize {
        self.x * self.y
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.x, self.y)
    }
}

impl From<Area> for Size {
    fn from(a: Area) -> Self {
        let n = a.normalize();
        Size {
            x: n.1.x.saturating_add(1).abs() as usize,
            y: n.1.y.saturating_add(1).abs() as usize,
        }
    }
}
