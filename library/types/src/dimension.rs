//! Dimension helpers with encoding and decoding to and from indexes.

use crate::{
    lib::*,
    point::{Point2, Point3},
};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
/// The kinds of errors that can occur for a `[DimensionError]`.
pub(crate) enum ErrorKind {
    /// If the coordinate or index is out of bounds.
    OutOfBounds,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use ErrorKind::*;
        match *self {
            OutOfBounds => write!(f, "out of bounds in the chunk"),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
/// An error type for dimensions which indicates that an error has occurred.
pub struct DimensionError(Box<ErrorKind>);

impl Display for DimensionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl Error for DimensionError {}

impl From<ErrorKind> for DimensionError {
    fn from(err: ErrorKind) -> DimensionError {
        DimensionError(Box::new(err))
    }
}

/// A dimension result which is of the type `Result<T, DimensionError>`.
pub type DimensionResult<T> = Result<T, DimensionError>;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
/// Dimensions of the 2nd kind.
pub struct Dimension2 {
    /// The width of this dimension.
    pub width: u32,
    /// The height of this dimension.
    pub height: u32,
}

impl Dimension2 {
    /// Constructs a new 2nd dimension.
    pub const fn new(width: u32, height: u32) -> Dimension2 {
        Dimension2 { width, height }
    }

    /// The total area of the dimension.
    pub fn area(&self) -> u32 {
        self.width * self.height
    }

    /// The minimum X value of this dimension.
    pub fn x_min(&self) -> i32 {
        -(self.width as i32) / 2
    }

    /// The minimum Y value of this dimension.
    pub fn y_min(&self) -> i32 {
        -(self.height as i32) / 2
    }

    /// The maximum X value of this dimension.
    pub fn x_max(&self) -> i32 {
        self.width as i32 / 2
    }

    /// The maximum Y value of this dimension.
    pub fn y_max(&self) -> i32 {
        self.height as i32 / 2
    }

    /// Returns the center of the `Map` as a `Vec2` `Chunk` coordinate.
    pub fn center(&self) -> Point2 {
        Point2::new((self.width / 2) as i32, (self.height / 2) as i32)
    }

    /// Checks if a coordinate is valid and inbounds.
    ///
    /// # Errors
    ///
    /// If the point does not exist in the dimensions, an error is returned.
    pub fn check_point(&self, point: Point2) -> DimensionResult<()> {
        if point.x > self.x_max()
            || point.y > self.y_max()
            || point.x < self.x_min()
            || point.y < self.y_min()
        {
            Err(ErrorKind::OutOfBounds.into())
        } else {
            Ok(())
        }
    }
    /// Checks if an index is valid and inbounds.
    ///
    /// # Errors
    ///
    /// If the point does not exist in the dimensions, an error is returned.

    pub fn check_index(&self, index: usize) -> DimensionResult<()> {
        if index > (self.width * self.height) as usize {
            Err(ErrorKind::OutOfBounds.into())
        } else {
            Ok(())
        }
    }
    /// Encodes a coordinate and returns an index value, unchecked.

    pub fn encode_point_unchecked(&self, point: Point2) -> usize {
        ((point.y * self.width as i32) + point.x) as usize
    }

    /// Encodes a coordinate and returns an index value.
    ///
    /// # Errors
    ///
    /// If the point does not exist in the dimensions, an error is returned.
    pub fn encode_point(&self, point: Point2) -> DimensionResult<usize> {
        self.check_point(point)?;
        Ok(self.encode_point_unchecked(point))
    }

    /// Decodes an index value and returns a coordinate, unchecked.
    pub fn decode_point_unchecked(&self, index: usize) -> Point2 {
        let y = index as i32 / self.height as i32;
        let x = index as i32 % self.width as i32;
        Point2::new(x, y)
    }

    /// Decodes an index value and returns a coordinate.
    ///
    /// # Errors
    ///
    /// If the point does not exist in the dimensions, an error is returned.
    pub fn decode_point(&self, index: usize) -> DimensionResult<Point2> {
        self.check_index(index)?;
        Ok(self.decode_point_unchecked(index))
    }
}

impl Display for Dimension2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl From<Dimension2> for Vec2 {
    fn from(dimension: Dimension2) -> Vec2 {
        Vec2::new(dimension.width as f32, dimension.height as f32)
    }
}

impl From<Extent3d> for Dimension2 {
    fn from(ext: Extent3d) -> Dimension2 {
        Dimension2::new(ext.width, ext.height)
    }
}

macro_rules! dimension2_glam_impl {
    ($vec: ty) => {
        impl From<$vec> for Dimension2 {
            fn from(vec: $vec) -> Dimension2 {
                Dimension2 {
                    width: vec.x as u32,
                    height: vec.y as u32,
                }
            }
        }

        impl From<&$vec> for Dimension2 {
            fn from(vec: &$vec) -> Dimension2 {
                Dimension2::from(*vec)
            }
        }
    };
}

dimension2_glam_impl!(Vec2);
dimension2_glam_impl!(Vec3);

macro_rules! dimension2_arr_impl {
    ($arr: ty) => {
        impl From<$arr> for Dimension2 {
            fn from(arr: $arr) -> Dimension2 {
                Dimension2 {
                    width: arr[0] as u32,
                    height: arr[1] as u32,
                }
            }
        }

        impl From<&$arr> for Dimension2 {
            fn from(arr: &$arr) -> Dimension2 {
                Dimension2::from(*arr)
            }
        }
    };
}

dimension2_arr_impl!([isize; 2]);
dimension2_arr_impl!([i64; 2]);
dimension2_arr_impl!([i32; 2]);
dimension2_arr_impl!([i16; 2]);
dimension2_arr_impl!([i8; 2]);
dimension2_arr_impl!([usize; 2]);
dimension2_arr_impl!([u64; 2]);
dimension2_arr_impl!([u32; 2]);
dimension2_arr_impl!([u16; 2]);
dimension2_arr_impl!([u8; 2]);

dimension2_arr_impl!([isize; 3]);
dimension2_arr_impl!([i64; 3]);
dimension2_arr_impl!([i32; 3]);
dimension2_arr_impl!([i16; 3]);
dimension2_arr_impl!([i8; 3]);
dimension2_arr_impl!([usize; 3]);
dimension2_arr_impl!([u64; 3]);
dimension2_arr_impl!([u32; 3]);
dimension2_arr_impl!([u16; 3]);
dimension2_arr_impl!([u8; 3]);

macro_rules! dimension2_tuple_impl {
    ($t: ty) => {
        impl From<$t> for Dimension2 {
            fn from(int: $t) -> Dimension2 {
                Dimension2 {
                    width: int.0 as u32,
                    height: int.1 as u32,
                }
            }
        }

        impl From<&$t> for Dimension2 {
            fn from(int: &$t) -> Dimension2 {
                Dimension2 {
                    width: int.0 as u32,
                    height: int.1 as u32,
                }
            }
        }
    };
}

dimension2_tuple_impl!((isize, isize));
dimension2_tuple_impl!((i64, i64));
dimension2_tuple_impl!((i32, i32));
dimension2_tuple_impl!((i16, i16));
dimension2_tuple_impl!((i8, i8));
dimension2_tuple_impl!((usize, usize));
dimension2_tuple_impl!((u64, u64));
dimension2_tuple_impl!((u32, u32));
dimension2_tuple_impl!((u16, u16));
dimension2_tuple_impl!((u8, u8));

dimension2_tuple_impl!((isize, isize, isize));
dimension2_tuple_impl!((i64, i64, i64));
dimension2_tuple_impl!((i32, i32, i32));
dimension2_tuple_impl!((i16, i16, i16));
dimension2_tuple_impl!((i8, i8, i8));
dimension2_tuple_impl!((usize, usize, usize));
dimension2_tuple_impl!((u64, u64, u64));
dimension2_tuple_impl!((u32, u32, u32));
dimension2_tuple_impl!((u16, u16, u16));
dimension2_tuple_impl!((u8, u8, u8));

impl Add for Dimension2 {
    type Output = Dimension2;

    fn add(self, rhs: Self) -> Self::Output {
        Dimension2::new(self.width + rhs.width, self.height + rhs.height)
    }
}

impl AddAssign for Dimension2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Dimension2::new(self.width + rhs.width, self.height + rhs.height)
    }
}

impl Div for Dimension2 {
    type Output = Dimension2;

    fn div(self, rhs: Self) -> Self::Output {
        Dimension2::new(self.width / rhs.width, self.height / rhs.height)
    }
}

impl DivAssign for Dimension2 {
    fn div_assign(&mut self, rhs: Self) {
        *self = Dimension2::new(self.width / rhs.width, self.height / rhs.height)
    }
}

impl Mul for Dimension2 {
    type Output = Dimension2;

    fn mul(self, rhs: Self) -> Self::Output {
        Dimension2::new(self.width * rhs.width, self.height * rhs.height)
    }
}

impl MulAssign for Dimension2 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = Dimension2::new(self.width * rhs.width, self.height * rhs.height)
    }
}

impl Sub for Dimension2 {
    type Output = Dimension2;

    fn sub(self, rhs: Self) -> Self::Output {
        Dimension2::new(self.width - rhs.width, self.height - rhs.height)
    }
}

impl SubAssign for Dimension2 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Dimension2::new(self.width - rhs.width, self.height - rhs.height)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
/// Dimensions of the 3rd kind.
pub struct Dimension3 {
    /// The width of this dimension.
    pub width: u32,
    /// The height of this dimension.
    pub height: u32,
    /// The depth of this dimension.
    pub depth: u32,
}

impl Dimension3 {
    /// Constructs a new 2nd dimension.
    pub const fn new(width: u32, height: u32, depth: u32) -> Dimension3 {
        Dimension3 {
            width,
            height,
            depth,
        }
    }

    /// The total area of the dimension.
    pub fn area(&self) -> u32 {
        self.width * self.height * self.depth
    }

    /// The maximum X value of this dimension.
    pub fn x_max(&self) -> u32 {
        self.width - 1
    }

    /// The maximum Y value of this dimension.
    pub fn y_max(&self) -> u32 {
        self.height - 1
    }

    /// The maximum Z value of this dimension.
    pub fn z_max(&self) -> u32 {
        self.depth - 1
    }

    /// Returns the center as a `Vec3`.
    pub fn center(&self) -> Point3 {
        Point3::new(
            self.width as i32 / 2,
            self.height as i32 / 2,
            self.depth as i32 / 2,
        )
    }

    /// Checks if a given coordinate is within bounds of the `Chunk`.
    ///
    /// # Errors
    ///
    /// If the point does not exist in the dimensions, an error is returned.
    pub fn check_point(&self, point: Point3) -> DimensionResult<()> {
        if point.x > self.width as i32
            || point.y > self.height as i32
            || point.z > self.depth as i32
        {
            Err(ErrorKind::OutOfBounds.into())
        } else {
            Ok(())
        }
    }

    /// Checks if a given index is within bounds of the `Chunk`.
    ///
    /// # Errors
    ///
    /// If the point does not exist in the dimensions, an error is returned.
    pub fn check_index(&self, index: usize) -> DimensionResult<()> {
        if index > (self.width * self.height * self.depth) as usize {
            Err(ErrorKind::OutOfBounds.into())
        } else {
            Ok(())
        }
    }

    /// Encodes a Vec3 coordinate to an usize index to use in the Tile vector, unchecked.
    pub fn encode_point_unchecked(&self, point: Point3) -> usize {
        ((point.z * self.width as i32 * self.height as i32)
            + (point.y * self.width as i32)
            + point.x) as usize
    }

    /// Encodes a `Vec3` coordinate to an `usize` index to use in the `Tile` vector.
    ///
    /// # Errors
    ///
    /// If the point does not exist in the dimensions, an error is returned.
    pub fn encode_point(&self, point: Point3) -> DimensionResult<usize> {
        self.check_point(point)?;
        Ok(self.encode_point_unchecked(point))
    }

    /// Decodes a Tile index and returns the coordinates in the Chunk, unchecked.
    ///
    /// # Errors
    ///
    /// If the point does not exist in the dimensions, an error is returned.
    pub fn decode_coord_unchecked(&self, index: usize) -> Point3 {
        let z = index as u32 / (self.width * self.height);
        let index = index as u32 - (z * self.width * self.height);
        let y = self.y_max() - (index / self.width);
        let x = index % self.width;
        Point3::new(x as i32, y as i32, z as i32)
    }

    /// Decodes a `Tile` index and returns the coordinates in the `Chunk`.
    ///
    /// # Errors
    ///
    /// If the point does not exist in the dimensions, an error is returned.

    pub fn decode_coord(&self, index: usize) -> DimensionResult<Point3> {
        self.check_index(index)?;
        Ok(self.decode_coord_unchecked(index))
    }
}

impl Display for Dimension3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}x{}x{}", self.width, self.height, self.depth)
    }
}

impl From<Dimension3> for Vec3 {
    fn from(dimension: Dimension3) -> Vec3 {
        Vec3::new(
            dimension.width as f32,
            dimension.height as f32,
            dimension.depth as f32,
        )
    }
}

impl From<Dimension3> for Dimension2 {
    fn from(dimension: Dimension3) -> Dimension2 {
        Dimension2::new(dimension.width, dimension.height)
    }
}

impl From<Extent3d> for Dimension3 {
    fn from(ext: Extent3d) -> Dimension3 {
        Dimension3::new(ext.width, ext.height, ext.depth)
    }
}

impl From<Dimension3> for Extent3d {
    fn from(dim: Dimension3) -> Extent3d {
        Extent3d::new(dim.width, dim.height, dim.depth)
    }
}

macro_rules! dimension3_glam_impl {
    ($vec: ty) => {
        impl From<$vec> for Dimension3 {
            fn from(vec: $vec) -> Dimension3 {
                Dimension3 {
                    width: vec.x as u32,
                    height: vec.y as u32,
                    depth: vec.z as u32,
                }
            }
        }

        impl From<&$vec> for Dimension3 {
            fn from(vec: &$vec) -> Dimension3 {
                Dimension3::from(*vec)
            }
        }
    };
}

dimension3_glam_impl!(Vec3);

macro_rules! dimension3_arr_impl {
    ($arr: ty) => {
        impl From<$arr> for Dimension3 {
            fn from(arr: $arr) -> Dimension3 {
                Dimension3 {
                    width: arr[0] as u32,
                    height: arr[1] as u32,
                    depth: arr[2] as u32,
                }
            }
        }

        impl From<&$arr> for Dimension3 {
            fn from(arr: &$arr) -> Dimension3 {
                Dimension3::from(*arr)
            }
        }
    };
}

dimension3_arr_impl!([isize; 3]);
dimension3_arr_impl!([i64; 3]);
dimension3_arr_impl!([i32; 3]);
dimension3_arr_impl!([i16; 3]);
dimension3_arr_impl!([i8; 3]);
dimension3_arr_impl!([usize; 3]);
dimension3_arr_impl!([u64; 3]);
dimension3_arr_impl!([u32; 3]);
dimension3_arr_impl!([u16; 3]);
dimension3_arr_impl!([u8; 3]);

macro_rules! dimension3_tuple_impl {
    ($t: ty) => {
        impl From<$t> for Dimension3 {
            fn from(int: $t) -> Dimension3 {
                Dimension3 {
                    width: int.0 as u32,
                    height: int.1 as u32,
                    depth: int.2 as u32,
                }
            }
        }

        impl From<&$t> for Dimension3 {
            fn from(int: &$t) -> Dimension3 {
                Dimension3 {
                    width: int.0 as u32,
                    height: int.1 as u32,
                    depth: int.2 as u32,
                }
            }
        }
    };
}

dimension3_tuple_impl!((isize, isize, isize));
dimension3_tuple_impl!((i64, i64, i64));
dimension3_tuple_impl!((i32, i32, i32));
dimension3_tuple_impl!((i16, i16, i16));
dimension3_tuple_impl!((i8, i8, i8));
dimension3_tuple_impl!((usize, usize, usize));
dimension3_tuple_impl!((u64, u64, u64));
dimension3_tuple_impl!((u32, u32, u32));
dimension3_tuple_impl!((u16, u16, u16));
dimension3_tuple_impl!((u8, u8, u8));

impl Add for Dimension3 {
    type Output = Dimension3;

    fn add(self, rhs: Self) -> Self::Output {
        Dimension3::new(
            self.width + rhs.width,
            self.height + rhs.height,
            self.depth + rhs.depth,
        )
    }
}

impl AddAssign for Dimension3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Dimension3::new(
            self.width + rhs.width,
            self.height + rhs.height,
            self.depth + rhs.depth,
        )
    }
}

impl Div for Dimension3 {
    type Output = Dimension3;

    fn div(self, rhs: Self) -> Self::Output {
        Dimension3::new(
            self.width / rhs.width,
            self.height / rhs.height,
            self.depth / rhs.depth,
        )
    }
}

impl DivAssign for Dimension3 {
    fn div_assign(&mut self, rhs: Self) {
        *self = Dimension3::new(
            self.width / rhs.width,
            self.height / rhs.height,
            self.depth / rhs.depth,
        )
    }
}

impl Mul for Dimension3 {
    type Output = Dimension3;

    fn mul(self, rhs: Self) -> Self::Output {
        Dimension3::new(
            self.width * rhs.width,
            self.height * rhs.height,
            self.depth * rhs.depth,
        )
    }
}

impl MulAssign for Dimension3 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = Dimension3::new(
            self.width * rhs.width,
            self.height * rhs.height,
            self.depth * rhs.depth,
        )
    }
}

impl Sub for Dimension3 {
    type Output = Dimension3;

    fn sub(self, rhs: Self) -> Self::Output {
        Dimension3::new(
            self.width - rhs.width,
            self.height - rhs.height,
            self.depth - rhs.depth,
        )
    }
}

impl SubAssign for Dimension3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Dimension3::new(
            self.width - rhs.width,
            self.height - rhs.height,
            self.depth - rhs.depth,
        )
    }
}
