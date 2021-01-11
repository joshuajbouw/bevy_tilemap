//! Points used for helping with coordinates.

use crate::lib::*;

/// A point which contains a X,Y coordinate.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Point2 {
    /// X value of a point.
    pub x: i32,
    /// Y value of a point.
    pub y: i32,
}

impl Point2 {
    /// Constructs a new point with a X,Y coordinate.
    pub fn new(x: i32, y: i32) -> Point2 {
        Point2 { x, y }
    }
}

impl Display for Point2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl From<&Point2> for Point2 {
    fn from(point: &Point2) -> Point2 {
        *point
    }
}

impl From<Point2> for Vec2 {
    fn from(point: Point2) -> Vec2 {
        Vec2::new(point.x as f32, point.y as f32)
    }
}

impl From<&Point2> for Vec2 {
    fn from(point: &Point2) -> Vec2 {
        Vec2::new(point.x as f32, point.y as f32)
    }
}

macro_rules! point2_glam_impl {
    ($vec: ty) => {
        impl From<$vec> for Point2 {
            fn from(vec: $vec) -> Point2 {
                Point2 {
                    x: vec.x as i32,
                    y: vec.y as i32,
                }
            }
        }

        impl From<&$vec> for Point2 {
            fn from(vec: &$vec) -> Point2 {
                Point2 {
                    x: vec.x as i32,
                    y: vec.y as i32,
                }
            }
        }
    };
}

point2_glam_impl!(Vec2);
point2_glam_impl!(Vec3);

macro_rules! point2_arr_impl {
    ($arr: ty) => {
        impl From<$arr> for Point2 {
            fn from(arr: $arr) -> Point2 {
                Point2 {
                    x: arr[0] as i32,
                    y: arr[1] as i32,
                }
            }
        }

        impl From<&$arr> for Point2 {
            fn from(arr: &$arr) -> Point2 {
                Point2 {
                    x: arr[0] as i32,
                    y: arr[1] as i32,
                }
            }
        }
    };
}

point2_arr_impl!([isize; 2]);
point2_arr_impl!([i64; 2]);
point2_arr_impl!([i32; 2]);
point2_arr_impl!([i16; 2]);
point2_arr_impl!([i8; 2]);
point2_arr_impl!([usize; 2]);
point2_arr_impl!([u64; 2]);
point2_arr_impl!([u32; 2]);
point2_arr_impl!([u16; 2]);
point2_arr_impl!([u8; 2]);

point2_arr_impl!([isize; 3]);
point2_arr_impl!([i64; 3]);
point2_arr_impl!([i32; 3]);
point2_arr_impl!([i16; 3]);
point2_arr_impl!([i8; 3]);
point2_arr_impl!([usize; 3]);
point2_arr_impl!([u64; 3]);
point2_arr_impl!([u32; 3]);
point2_arr_impl!([u16; 3]);
point2_arr_impl!([u8; 3]);

macro_rules! point2_tuple_impl {
    ($t: ty) => {
        impl From<$t> for Point2 {
            fn from(int: $t) -> Point2 {
                Point2 {
                    x: int.0 as i32,
                    y: int.1 as i32,
                }
            }
        }

        impl From<&$t> for Point2 {
            fn from(int: &$t) -> Point2 {
                Point2 {
                    x: int.0 as i32,
                    y: int.1 as i32,
                }
            }
        }
    };
}

point2_tuple_impl!((isize, isize));
point2_tuple_impl!((i64, i64));
point2_tuple_impl!((i32, i32));
point2_tuple_impl!((i16, i16));
point2_tuple_impl!((i8, i8));
point2_tuple_impl!((usize, usize));
point2_tuple_impl!((u64, u64));
point2_tuple_impl!((u32, u32));
point2_tuple_impl!((u16, u16));
point2_tuple_impl!((u8, u8));

point2_tuple_impl!((isize, isize, isize));
point2_tuple_impl!((i64, i64, i64));
point2_tuple_impl!((i32, i32, i32));
point2_tuple_impl!((i16, i16, i16));
point2_tuple_impl!((i8, i8, i8));
point2_tuple_impl!((usize, usize, usize));
point2_tuple_impl!((u64, u64, u64));
point2_tuple_impl!((u32, u32, u32));
point2_tuple_impl!((u16, u16, u16));
point2_tuple_impl!((u8, u8, u8));

impl Add for Point2 {
    type Output = Point2;

    fn add(self, rhs: Self) -> Self::Output {
        Point2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Point2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Point2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Div for Point2 {
    type Output = Point2;

    fn div(self, rhs: Self) -> Self::Output {
        Point2 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl DivAssign for Point2 {
    fn div_assign(&mut self, rhs: Self) {
        *self = Point2 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl Mul for Point2 {
    type Output = Point2;

    fn mul(self, rhs: Self) -> Self::Output {
        Point2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl MulAssign for Point2 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = Point2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Neg for Point2 {
    type Output = Point2;

    fn neg(self) -> Self::Output {
        Point2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Sub for Point2 {
    type Output = Point2;

    fn sub(self, rhs: Self) -> Self::Output {
        Point2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Point2 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Point2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// A point which contains a X,Y,Z coordinate.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Point3 {
    /// X value of a point.
    pub x: i32,
    /// Y value of a point.
    pub y: i32,
    /// Z value of a point.
    pub z: i32,
}

impl Point3 {
    /// Constructs a new point with a X,Y,Z coordinate.
    pub fn new(x: i32, y: i32, z: i32) -> Point3 {
        Point3 { x, y, z }
    }

    /// The X,Y coordinate of the point as a [`Point2`](Point2).
    pub fn xy(&self) -> Point2 {
        Point2::new(self.x, self.y)
    }
}

impl Display for Point3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl From<Point3> for Point2 {
    fn from(point: Point3) -> Point2 {
        Point2::new(point.x, point.y)
    }
}

impl From<&Point3> for Point2 {
    fn from(point: &Point3) -> Point2 {
        Point2::new(point.x, point.y)
    }
}

impl From<&Point3> for Point3 {
    fn from(point: &Point3) -> Point3 {
        *point
    }
}

impl From<Point3> for Vec3 {
    fn from(point: Point3) -> Vec3 {
        Vec3::new(point.x as f32, point.y as f32, point.z as f32)
    }
}

impl From<&Point3> for Vec3 {
    fn from(point: &Point3) -> Vec3 {
        Vec3::new(point.x as f32, point.y as f32, point.z as f32)
    }
}

macro_rules! point3_glam_impl {
    ($vec: ty) => {
        impl From<$vec> for Point3 {
            fn from(vec: $vec) -> Point3 {
                Point3 {
                    x: vec.x as i32,
                    y: vec.y as i32,
                    z: vec.z as i32,
                }
            }
        }

        impl From<&$vec> for Point3 {
            fn from(vec: &$vec) -> Point3 {
                Point3 {
                    x: vec.x as i32,
                    y: vec.y as i32,
                    z: vec.z as i32,
                }
            }
        }
    };
}

point3_glam_impl!(Vec3);

macro_rules! point3_arr_impl {
    ($vec: ty) => {
        impl From<$vec> for Point3 {
            fn from(vec: $vec) -> Point3 {
                Point3 {
                    x: vec[0] as i32,
                    y: vec[1] as i32,
                    z: vec[2] as i32,
                }
            }
        }

        impl From<&$vec> for Point3 {
            fn from(vec: &$vec) -> Point3 {
                Point3 {
                    x: vec[0] as i32,
                    y: vec[1] as i32,
                    z: vec[2] as i32,
                }
            }
        }
    };
}

point3_arr_impl!([isize; 3]);
point3_arr_impl!([i64; 3]);
point3_arr_impl!([i32; 3]);
point3_arr_impl!([i16; 3]);
point3_arr_impl!([i8; 3]);
point3_arr_impl!([usize; 3]);
point3_arr_impl!([u64; 3]);
point3_arr_impl!([u32; 3]);
point3_arr_impl!([u16; 3]);
point3_arr_impl!([u8; 3]);

macro_rules! point3_impl {
    ($t: ty) => {
        impl From<$t> for Point3 {
            fn from(int: $t) -> Point3 {
                Point3 {
                    x: int.0 as i32,
                    y: int.1 as i32,
                    z: int.2 as i32,
                }
            }
        }

        impl From<&$t> for Point3 {
            fn from(int: &$t) -> Point3 {
                Point3 {
                    x: int.0 as i32,
                    y: int.1 as i32,
                    z: int.2 as i32,
                }
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

impl Add for Point3 {
    type Output = Point3;

    fn add(self, rhs: Self) -> Self::Output {
        Point3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Point3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Point3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Div for Point3 {
    type Output = Point3;

    fn div(self, rhs: Self) -> Self::Output {
        Point3 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl DivAssign for Point3 {
    fn div_assign(&mut self, rhs: Self) {
        *self = Point3 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl Mul for Point3 {
    type Output = Point3;

    fn mul(self, rhs: Self) -> Self::Output {
        Point3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl MulAssign for Point3 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = Point3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Neg for Point3 {
    type Output = Point3;

    fn neg(self) -> Self::Output {
        Point3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Sub for Point3 {
    type Output = Point3;

    fn sub(self, rhs: Self) -> Self::Output {
        Point3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl SubAssign for Point3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Point3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
