use std::fmt;

use serde::{Deserialize, Serialize};

use super::{
    point::{Point, EMPTY_POINT},
    Size,
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
/// A representation between the space of two points. First point is always top-left. The second
/// point should always be larger. Generally, the boundaries are inclusive. To have the boundaries
/// inclusive, ie, for counting the number of cells: convert into a [Size].
pub struct Area(pub Point, pub Point);

impl Area {
    /// Calculate the box area around a given point. Radius of 1 => 3x3 Area
    pub fn around_point(p: Point, radius: usize) -> Area {
        Area(
            Point {
                x: p.x.saturating_sub(radius as i32),
                y: p.y.saturating_sub(radius as i32),
            },
            Point {
                x: p.x.saturating_add(radius as i32),
                y: p.y.saturating_add(radius as i32),
            },
        )
    }

    pub fn contains(&self, p: Point) -> bool {
        let Area(min, max) = self;
        p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y
    }

    pub fn normalize(&self) -> Area {
        let Area(min, max) = self;
        Area(
            EMPTY_POINT,
            Point {
                x: max.x.saturating_sub(min.x),
                y: max.y.saturating_sub(min.y),
            },
        )
    }

    pub fn point_from_pos(self, pos: usize) -> Point {
        let n = Size::from(self);
        Point {
            x: (pos % n.x) as i32,
            y: (pos / n.x) as i32,
        }
    }

    pub fn intersecting_area(&self, a: Area) -> Area {
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

    pub fn clamp_point(&self, p: Point) -> Point {
        Point {
            x: p.x.max(self.0.x).min(self.1.x),
            y: p.y.max(self.0.y).min(self.1.y),
        }
    }
}

impl fmt::Display for Area {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl From<Size> for Area {
    fn from(value: Size) -> Self {
        Area::from(Point::from(value))
    }
}

impl From<Point> for Area {
    fn from(value: Point) -> Self {
        Area(EMPTY_POINT, value)
    }
}
