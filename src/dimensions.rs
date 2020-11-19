use crate::lib::*;
use crate::point::{Point2, Point3};

#[derive(Clone, Copy, PartialEq)]
/// The kinds of errors that can occur for a `[DimensionError]`.
pub(crate) enum ErrorKind {
    /// If the coordinate or index is out of bounds.
    OutOfBounds,
    /// The minimum constraint is larger than the maximum.
    MinLargerThanMax(i32, i32),
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use ErrorKind::*;
        match *self {
            OutOfBounds => write!(f, "out of bounds in the chunk"),
            MinLargerThanMax(min, max) => write!(f, "minimum constraint {} larger than maximum {}", min, max),
        }
    }
}

#[derive(Clone, PartialEq)]
/// A MapError indicates that an error with the `[Chunk]` has occurred.
pub(crate) struct DimensionError(Box<ErrorKind>);

impl Debug for DimensionError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.fmt(f)
    }
}

impl From<ErrorKind> for DimensionError {
    fn from(err: ErrorKind) -> DimensionError {
        DimensionError::new(err)
    }
}

impl DimensionError {
    /// Creates a new `MapError`.
    pub fn new(kind: ErrorKind) -> DimensionError {
        DimensionError(Box::new(kind))
    }

    /// Returns the underlying error kind `ErrorKind`.
    pub fn kind(&self) -> ErrorKind {
        *self.0
    }
}

/// A chunk result.
pub(crate) type DimensionResult<T> = Result<T, DimensionError>;


pub(crate) struct Dimensions2 {
    width: u32,
    height: u32,
}

impl Dimensions2 {
    pub(crate) fn new(width: u32, height: u32) -> Dimensions2 {
        Dimensions2 {
            width,
            height,
        }
    }

    /// The width of this dimension.
    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    /// The height of this dimension.
    pub(crate) fn height(&self) -> u32 {
        self.height
    }

    /// The maximum X value of this dimension.
    pub(crate) fn x_max(&self) -> u32 {
        self.width - 1
    }

    /// The maximum Y value of this dimension.
    pub(crate) fn y_max(&self) -> u32 {
        self.height - 1
    }

    /// Returns the center of the `Map` as a `Vec2` `Chunk` coordinate.
    pub(crate) fn center(&self) -> Point2 {
        Point2::new((self.width / 2) as i32, (self.height / 2) as i32)
    }

    /// Checks if a coordinate is valid and inbounds.
    pub(crate) fn check_point(&self, point: &Point2) -> DimensionResult<()> {
        if point.x() > self.x_max() as i32
            || point.y() > self.y_max() as i32
        {
            Err(ErrorKind::OutOfBounds.into())
        } else {
            Ok(())
        }
    }

    /// Checks if an index is valid and inbounds.
    pub(crate) fn check_index(&self, index: usize) -> DimensionResult<()> {
        if index > (self.width * self.height) as usize {
            Err(ErrorKind::OutOfBounds.into())
        } else {
            Ok(())
        }
    }

    /// Encodes a coordinate and returns an index value, unchecked.
    pub(crate) fn encode_point_unchecked(&self, point: &Point2) -> usize {
        ((point.y() * self.width as i32) + point.x()) as usize
    }

    /// Encodes a coordinate and returns an index value.
    pub(crate) fn encode_point(&self, point: &Point2) -> DimensionResult<usize> {
        self.check_point(point)?;
        Ok(self.encode_point_unchecked(point))
    }

    /// Decodes an index value and returns a coordinate, unchecked.
    pub(crate)fn decode_point_unchecked(&self, index: usize) -> Point2 {
        let y = index as i32 / self.height as i32;
        let x = index as i32 % self.width as i32;
        Point2::new(x, y)
    }

    /// Decodes an index value and returns a coordinate.
    pub(crate)fn decode_point(&self, index: usize) -> DimensionResult<Point2> {
        self.check_index(index)?;
        Ok(self.decode_point_unchecked(index))
    }
}

pub(crate) struct Dimensions3 {
    width: u32,
    height: u32,
    depth: u32,
}

/// Trait methods that have to do with the 3rd dimension.
impl Dimensions3 {
    pub(crate) fn new(width: u32, height: u32, depth: u32) -> Dimensions3 {
        Dimensions3 {
            width,
            height,
            depth
        }
    }

    /// The width of this dimension.
    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    /// The height of this dimension.
    pub(crate) fn height(&self) -> u32 {
        self.height
    }

    /// The depth of this dimension.
    pub(crate) fn depth(&self) -> u32 {
        self.depth
    }

    /// The maximum X value of this dimension.
    pub(crate) fn x_max(&self) -> u32 {
        self.width - 1
    }

    /// The maximum Y value of this dimension.
    pub(crate) fn y_max(&self) -> u32 {
        self.height - 1
    }

    /// The maximum Z value of this dimension.
    pub(crate) fn z_max(&self) -> u32 {
        self.depth - 1
    }

    /// Returns the center as a `Vec3`.
    pub(crate) fn center(&self) -> Point3 {
        Point3::new(
            self.width as i32 / 2,
            self.height as i32 / 2,
            self.depth as i32 / 2,
        )
    }

    /// Checks if a given coordinate is within bounds of the `Chunk`.
    pub(crate) fn check_point(&self, point: Point3) -> DimensionResult<()> {
    if point.x() > self.width as i32
        || point.y() > self.height as i32
        || point.z() > self.depth as i32
        {
            Err(ErrorKind::OutOfBounds.into())
        } else {
            Ok(())
        }
    }

    /// Checks if a given index is within bounds of the `Chunk`.
    pub(crate) fn check_index(&self, index: usize) -> DimensionResult<()> {
        if index > (self.width * self.height * self.depth) as usize {
            Err(ErrorKind::OutOfBounds.into())
        } else {
            Ok(())
        }
    }

    /// Encodes a Vec3 coordinate to an usize index to use in the Tile vector, unchecked.
    pub(crate) fn encode_point_unchecked(&self, point: Point3) -> usize {
        ((point.z() * self.width as i32 * self.height as i32)
            + (point.y() * self.width as i32)
            + point.x()) as usize
    }

    /// Encodes a `Vec3` coordinate to an `usize` index to use in the `Tile` vector.
    pub(crate) fn encode_point(&self, point: Point3) -> DimensionResult<usize> {
        self.check_point(point)?;
        Ok(self.encode_point_unchecked(point))
    }

    /// Decodes a Tile index and returns the coordinates in the Chunk, unchecked.
    pub(crate) fn decode_coord_unchecked(&self, index: usize) -> Point3 {
        let z = index as u32 / (self.width * self.height);
        let index = index as u32 - (z * self.width * self.height);
        let y = self.y_max() - (index / self.width);
        let x = index % self.width;
        Point3::new(x as i32, y as i32, z as i32)
    }

    /// Decodes a `Tile` index and returns the coordinates in the `Chunk`.
    pub(crate) fn decode_coord(&self, index: usize) -> DimensionResult<Point3> {
        self.check_index(index)?;
        Ok(self.decode_coord_unchecked(index))
    }
}

#[deprecated(
    since = "0.2.0",
    note = "please use do not use, will be removed by v0.3.0"
)]
#[doc(hidden)]
pub mod deprecated {
    use crate::lib::*;

    #[derive(Clone, Copy, PartialEq)]
    /// The kinds of errors that can occur for a `[DimensionError]`.
    pub enum ErrorKind {
        /// If the coordinate or index is out of bounds.
        OutOfBounds,
    }

    impl Debug for ErrorKind {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            use ErrorKind::*;
            match *self {
                OutOfBounds => write!(f, "out of bounds in the chunk"),
            }
        }
    }

    #[derive(Clone, PartialEq)]
    /// A MapError indicates that an error with the `[Chunk]` has occurred.
    pub struct DimensionError(Box<ErrorKind>);

    impl Debug for DimensionError {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            self.0.fmt(f)
        }
    }

    impl From<ErrorKind> for DimensionError {
        fn from(err: ErrorKind) -> DimensionError {
            DimensionError::new(err)
        }
    }

    impl DimensionError {
        /// Creates a new `MapError`.
        pub fn new(kind: ErrorKind) -> DimensionError {
            DimensionError(Box::new(kind))
        }

        /// Returns the underlying error kind `ErrorKind`.
        pub fn kind(&self) -> ErrorKind {
            *self.0
        }
    }

    /// A chunk result.
    pub type DimensionResult<T> = Result<T, DimensionError>;

    /// Trait methods that have to do with the 2nd dimension.
    pub trait Dimensions2 {
        /// A `Vec2` containing the dimensions.
        fn dimensions(&self) -> Vec2;

        /// The width of this dimension.
        fn width(&self) -> f32 {
            self.dimensions().x()
        }

        /// The height of this dimension.
        fn height(&self) -> f32 {
            self.dimensions().y()
        }

        /// The minimum X value of this dimension.
        fn min_x(&self) -> f32 {
            0.
        }

        /// The minimum Y value of this dimension.
        fn min_y(&self) -> f32 {
            0.
        }

        /// The maximum X value of this dimension.
        fn max_x(&self) -> f32 {
            self.dimensions().x() - 1.
        }

        /// The maximum Y value of this dimension.
        fn max_y(&self) -> f32 {
            self.dimensions().y() - 1.
        }

        /// Returns the center of the `Map` as a `Vec2` `Chunk` coordinate.
        fn center(&self) -> Vec2 {
            Vec2::new(self.dimensions().x() / 2., self.dimensions().y() / 2.)
        }

        /// Checks if a coordinate is valid and inbounds.
        fn check_coord(&self, coord: &Vec2) -> DimensionResult<()> {
            if coord.x() > self.max_x()
                || coord.y() > self.max_y()
                || coord.x() < self.min_x()
                || coord.y() < self.min_y()
            {
                Err(ErrorKind::OutOfBounds.into())
            } else {
                Ok(())
            }
        }

        /// Checks if an index is valid and inbounds.
        fn check_index(&self, idx: usize) -> DimensionResult<()> {
            // NOTE: I think this should have a min case?
            if idx > (self.dimensions().x() * self.dimensions().y()) as usize {
                Err(ErrorKind::OutOfBounds.into())
            } else {
                Ok(())
            }
        }

        /// Encodes a coordinate and returns an index value, unchecked.
        fn encode_coord_unchecked(&self, coord: &Vec2) -> usize {
            ((coord.y() * self.dimensions().x()) + coord.x()) as usize
        }

        /// Encodes a coordinate and returns an index value.
        fn encode_coord(&self, coord: &Vec2) -> DimensionResult<usize> {
            self.check_coord(coord)?;
            Ok(self.encode_coord_unchecked(coord))
        }

        /// Decodes an index value and returns a coordinate, unchecked.
        fn decode_coord_unchecked(&self, idx: usize) -> Vec2 {
            let y = idx / self.dimensions().y() as usize;
            let x = idx % self.dimensions().x() as usize;
            Vec2::new(x as f32, y as f32)
        }

        /// Decodes an index value and returns a coordinate.
        fn decode_coord(&self, idx: usize) -> DimensionResult<Vec2> {
            self.check_index(idx)?;
            Ok(self.decode_coord_unchecked(idx))
        }
    }

    impl Dimensions2 for Vec2 {
        fn dimensions(&self) -> Vec2 {
            *self
        }
    }

    /// Trait methods that have to do with the 3rd dimension.
    pub trait Dimensions3 {
        /// A `Vec3` containing the dimensions.
        fn dimensions(&self) -> Vec3;

        /// The width of this dimension.
        fn width(&self) -> f32 {
            self.dimensions().x()
        }

        /// The height of this dimension.
        fn height(&self) -> f32 {
            self.dimensions().y()
        }

        /// The depth of this dimension.
        fn depth(&self) -> f32 {
            self.dimensions().z()
        }

        /// The minimum X value of this dimension.
        fn min_x(&self) -> f32 {
            0.
        }

        /// The minimum Y value of this dimension.
        fn min_y(&self) -> f32 {
            0.
        }

        /// The minimum Z value of this dimension.
        fn min_z(&self) -> f32 {
            0.
        }

        /// The maximum X value of this dimension.
        fn max_x(&self) -> f32 {
            self.dimensions().x() - 1.
        }

        /// The maximum Y value of this dimension.
        fn max_y(&self) -> f32 {
            self.dimensions().y() - 1.
        }

        /// The maximum Z value of this dimension.
        fn max_z(&self) -> f32 {
            self.dimensions().z() - 1.
        }

        /// Returns the center as a `Vec3`.
        fn center(&self) -> Vec3 {
            Vec3::new(
                self.dimensions().x() / 2.,
                self.dimensions().y() / 2.,
                self.dimensions().z() / 2.,
            )
        }

        /// Checks if a given coordinate is within bounds of the `Chunk`.
        fn check_coord(&self, coord: &Vec3) -> DimensionResult<()> {
            if coord.x() > self.dimensions().x()
                || coord.y() > self.dimensions().y()
                || coord.z() > self.dimensions().z()
            {
                Err(ErrorKind::OutOfBounds.into())
            } else {
                Ok(())
            }
        }

        /// Checks if a given index is within bounds of the `Chunk`.
        fn check_index(&self, idx: usize) -> DimensionResult<()> {
            if idx > (self.dimensions().x() * self.dimensions().y() * self.dimensions().z()) as usize {
                Err(ErrorKind::OutOfBounds.into())
            } else {
                Ok(())
            }
        }

        /// Encodes a Vec3 coordinate to an usize index to use in the Tile vector, unchecked.
        fn encode_coord_unchecked(&self, coord: &Vec3) -> usize {
            ((coord.z() * self.dimensions().x() * self.dimensions().y())
                + (coord.y() * self.dimensions().x())
                + coord.x()) as usize
        }

        /// Encodes a `Vec3` coordinate to an `usize` index to use in the `Tile` vector.
        fn encode_coord(&self, coord: &Vec3) -> DimensionResult<usize> {
            self.check_coord(coord)?;
            Ok(self.encode_coord_unchecked(coord))
        }

        /// Decodes a Tile index and returns the coordinates in the Chunk, unchecked.
        fn decode_coord_unchecked(&self, idx: usize) -> Vec3 {
            let z = idx as f32 / (self.dimensions().x() * self.dimensions().y());
            let idx = idx as f32 - (z * self.dimensions().x() * self.dimensions().y());
            let y = self.dimensions().y() - 1. - (idx / self.dimensions().x());
            let x = idx % self.dimensions().x();
            Vec3::new(x, y, z)
        }

        /// Decodes a `Tile` index and returns the coordinates in the `Chunk`.
        fn decode_coord(&self, idx: usize) -> DimensionResult<Vec3> {
            self.check_index(idx)?;
            Ok(self.decode_coord_unchecked(idx))
        }
    }

    impl Dimensions3 for Vec3 {
        fn dimensions(&self) -> Vec3 {
            *self
        }
    }
}