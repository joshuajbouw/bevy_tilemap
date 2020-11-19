use crate::lib::*;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point2(i32, i32);

impl Point2 {
    pub fn new(x: i32, y: i32) -> Point2 {
        Point2(x, y)
    }

    pub fn x(&self) -> i32 {
        self.0
    }

    pub fn y(&self) -> i32 {
        self.1
    }
}

impl From<(i32, i32)> for Point2 {
    fn from(point2: (i32, i32)) -> Point2 {
        Point2(point2.0, point2.1)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point3(i32, i32, i32);

impl Point3 {
    pub fn new(x: i32, y: i32, z: i32) -> Point3 {
        Point3(x, y, z)
    }

    pub fn x(&self) -> i32 {
        self.0
    }

    pub fn y(&self) -> i32 {
        self.1
    }

    pub fn z(&self) -> i32 {
        self.2
    }

    pub fn xy(&self) -> Point2 {
        Point2::new(self.0, self.1)
    }
}

impl From<(i32, i32, i32)> for Point3 {
    fn from(point3: (i32, i32, i32)) -> Point3 {
        Point3(point3.0, point3.1, point3.2)
    }
}

impl From<&(i32, i32, i32)> for Point3 {
    fn from(points: &(i32, i32, i32)) -> Point3 {
        Point3::from(*points)
    }
}

impl From<Point3> for Point2 {
    fn from(point: Point3) -> Point2 {
        Point2::new(point.x(), point.y())
    }
}
