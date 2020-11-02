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

/// Trait methods that have to do with the 3rd dimension.
pub trait Dimensions3 {
    /// A `Vec3` containing the dimensions.
    fn dimensions(&self) -> Vec3;

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
