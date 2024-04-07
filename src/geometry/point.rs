use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    ops::Add,
};

use super::Size;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
/// A point in a 2D space
///
/// Note: Points are positive only
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub const EMPTY_POINT: Point = Point { x: 0, y: 0 };

impl Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl From<Size> for Point {
    fn from(value: Size) -> Self {
        Point {
            x: value.x.saturating_sub(1) as i32,
            y: value.y.saturating_sub(1) as i32,
        }
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
