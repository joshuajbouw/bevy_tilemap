use crate::lib::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub(crate) struct Point2(i32, i32);

impl Point2 {
    pub(crate) fn new(x: i32, y: i32) -> Point2 {
        Point2(x, y)
    }

    pub(crate) fn x(&self) -> i32 {
        self.0
    }

    pub(crate) fn y(&self) -> i32 {
        self.1
    }
}

impl Display for Point2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl From<&(i32, i32)> for Point2 {
    fn from(point2: &(i32, i32)) -> Point2 {
        Point2(point2.0, point2.1)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub(crate) struct Point3(i32, i32, i32);

impl Point3 {
    pub(crate) fn new(x: i32, y: i32, z: i32) -> Point3 {
        Point3(x, y, z)
    }

    pub(crate) fn x(&self) -> i32 {
        self.0
    }

    pub(crate) fn y(&self) -> i32 {
        self.1
    }

    pub(crate) fn z(&self) -> i32 {
        self.2
    }

    pub(crate) fn xy(&self) -> Point2 {
        Point2::new(self.0, self.1)
    }
}

impl Display for Point3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({}, {}, {})", self.0, self.1, self.2)
    }
}

impl From<(i32, i32, i32)> for Point3 {
    fn from(point3: (i32, i32, i32)) -> Point3 {
        Point3(point3.0, point3.1, point3.2)
    }
}

impl From<&(i32, i32, i32)> for Point3 {
    fn from(point3: &(i32, i32, i32)) -> Point3 {
        Point3(point3.0, point3.1, point3.2)
    }
}

impl From<&Point3> for Point2 {
    fn from(point: &Point3) -> Point2 {
        Point2::new(point.x(), point.y())
    }
}
