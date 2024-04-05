use serde::{Deserialize, Serialize};
use std::fmt;

use super::Size;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
/// A point in a 2D space
///
/// Note: Points are positive only
pub struct Point {
    pub x: usize,
    pub y: usize,
}

pub const EMPTY_POINT: Point = Point { x: 0, y: 0 };

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl From<Size> for Point {
    fn from(value: Size) -> Self {
        Point {
            x: value.x.saturating_sub(1),
            y: value.y.saturating_sub(1),
        }
    }
}

impl std::ops::Add for Point {
    type Output = Point;
    fn add(self, p: Point) -> Point {
        Point {
            x: self.x.saturating_add(p.x),
            y: self.y.saturating_add(p.y),
        }
    }
}
