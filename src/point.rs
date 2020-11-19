use crate::lib::*;

/// A point which contains a X,Y coordinate.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Point2(i32, i32);

impl Point2 {
    /// Constructs a new point with a X,Y coordinate.
    pub fn new(x: i32, y: i32) -> Point2 {
        Point2(x, y)
    }

    /// The X coordinate of the point.
    pub fn x(&self) -> i32 {
        self.0
    }

    /// The Y coordinate of the point.
    pub fn y(&self) -> i32 {
        self.1
    }
}

impl Display for Point2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({}, {})", self.0, self.1)
    }
}

macro_rules! point2_impl {
    ($t: ty) => {
        impl From<$t> for Point2 {
            fn from(int: $t) -> Point2 {
                Point2(int.0 as i32, int.1 as i32)
            }
        }

        impl From<&$t> for Point2 {
            fn from(int: &$t) -> Point2 {
                Point2(int.0 as i32, int.1 as i32)
            }
        }
    };
}

point2_impl!((isize, isize));
point2_impl!((i64, i64));
point2_impl!((i32, i32));
point2_impl!((i16, i16));
point2_impl!((i8, i8));
point2_impl!((usize, usize));
point2_impl!((u64, u64));
point2_impl!((u32, u32));
point2_impl!((u16, u16));
point2_impl!((u8, u8));

/// A point which contains a X,Y,Z coordinate.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Point3(i32, i32, i32);

impl Point3 {
    /// Constructs a new point with a X,Y,Z coordinate.
    pub fn new(x: i32, y: i32, z: i32) -> Point3 {
        Point3(x, y, z)
    }

    /// The X coordinate of the point.
    pub fn x(&self) -> i32 {
        self.0
    }

    /// The Y coordinate of the point.
    pub fn y(&self) -> i32 {
        self.1
    }

    /// The Z coordinate of the point.
    pub fn z(&self) -> i32 {
        self.2
    }

    /// The X,Y coordinate of the point as a [`Point2`](Point2).
    pub fn xy(&self) -> Point2 {
        Point2::new(self.0, self.1)
    }
}

impl Display for Point3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({}, {}, {})", self.0, self.1, self.2)
    }
}

impl From<&Point3> for Point2 {
    fn from(point: &Point3) -> Point2 {
        Point2::new(point.x(), point.y())
    }
}

macro_rules! point3_impl {
    ($t: ty) => {
        impl From<$t> for Point3 {
            fn from(int: $t) -> Point3 {
                Point3(int.0 as i32, int.1 as i32, int.2 as i32)
            }
        }

        impl From<&$t> for Point3 {
            fn from(int: &$t) -> Point3 {
                Point3(int.0 as i32, int.1 as i32, int.2 as i32)
            }
        }
    };
}

point3_impl!((isize, isize, isize));
point3_impl!((i64, i64, i64));
point3_impl!((i32, i32, i32));
point3_impl!((i16, i16, i16));
point3_impl!((i8, i8, i8));
point3_impl!((usize, usize, usize));
point3_impl!((u64, u64, u64));
point3_impl!((u32, u32, u32));
point3_impl!((u16, u16, u16));
point3_impl!((u8, u8, u8));
