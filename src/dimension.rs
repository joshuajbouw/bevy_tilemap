// NOTE: There is a lot of code in here commented out. This was intentional.
// Why? Because these all likely will end up in their own crate and it would
// not make sense to get rid of that code.

use crate::{lib::*, point::Point2};

#[derive(Clone, Copy, PartialEq)]
/// The kinds of errors that can occur for a `[DimensionError]`.
pub(crate) enum ErrorKind {
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
        DimensionError(Box::new(err))
    }
}

/// A chunk result.
pub type DimensionResult<T> = Result<T, DimensionError>;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Dimension2 {
    width: u32,
    height: u32,
}

impl Dimension2 {
    pub(crate) fn new(width: u32, height: u32) -> Dimension2 {
        Dimension2 { width, height }
    }

    /// The width of this dimension.
    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    /// The height of this dimension.
    pub(crate) fn height(&self) -> u32 {
        self.height
    }

    pub(crate) fn area(&self) -> u32 {
        self.width * self.height
    }

    /// The maximum X value of this dimension.
    pub(crate) fn x_max(&self) -> u32 {
        self.width - 1
    }

    /// The maximum Y value of this dimension.
    pub(crate) fn y_max(&self) -> u32 {
        self.height - 1
    }

    // /// Returns the center of the `Map` as a `Vec2` `Chunk` coordinate.
    // pub(crate) fn center(&self) -> Point2 {
    //     Point2::new((self.width / 2) as i32, (self.height / 2) as i32)
    // }

    /// Checks if a coordinate is valid and inbounds.
    pub(crate) fn check_point(&self, point: Point2) -> DimensionResult<()> {
        if point.x() > self.x_max() as i32 || point.y() > self.y_max() as i32 {
            Err(ErrorKind::OutOfBounds.into())
        } else {
            Ok(())
        }
    }

    // /// Checks if an index is valid and inbounds.
    // pub(crate) fn check_index(&self, index: usize) -> DimensionResult<()> {
    //     if index > (self.width * self.height) as usize {
    //         Err(ErrorKind::OutOfBounds.into())
    //     } else {
    //         Ok(())
    //     }
    // }

    /// Encodes a coordinate and returns an index value, unchecked.
    pub(crate) fn encode_point_unchecked(&self, point: Point2) -> usize {
        ((point.y() * self.width as i32) + point.x()) as usize
    }

    // /// Encodes a coordinate and returns an index value.
    // pub(crate) fn encode_point(&self, point: Point2) -> DimensionResult<usize> {
    //     self.check_point(point)?;
    //     Ok(self.encode_point_unchecked(point))
    // }

    // /// Decodes an index value and returns a coordinate, unchecked.
    // pub(crate) fn decode_point_unchecked(&self, index: usize) -> Point2 {
    //     let y = index as i32 / self.height as i32;
    //     let x = index as i32 % self.width as i32;
    //     Point2::new(x, y)
    // }

    // /// Decodes an index value and returns a coordinate.
    // pub(crate) fn decode_point(&self, index: usize) -> DimensionResult<Point2> {
    //     self.check_index(index)?;
    //     Ok(self.decode_point_unchecked(index))
    // }
}

impl From<(u32, u32)> for Dimension2 {
    fn from(tuple: (u32, u32)) -> Self {
        Dimension2 {
            width: tuple.0,
            height: tuple.1,
        }
    }
}

impl From<Dimension2> for (u32, u32) {
    fn from(dimensions: Dimension2) -> (u32, u32) {
        (dimensions.width(), dimensions.height())
    }
}

impl From<Dimension2> for Vec2 {
    fn from(point: Dimension2) -> Vec2 {
        Vec2::new(point.width() as f32, point.height() as f32)
    }
}

// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// #[derive(Debug, Copy, Clone)]
// pub struct Dimension3 {
//     width: u32,
//     height: u32,
//     depth: u32,
// }
//
// /// Trait methods that have to do with the 3rd dimension.
// impl Dimension3 {
//     pub(crate) fn new(width: u32, height: u32, depth: u32) -> Dimension3 {
//         Dimension3 {
//             width,
//             height,
//             depth,
//         }
//     }
//
//     /// The width of this dimension.
//     pub(crate) fn width(&self) -> u32 {
//         self.width
//     }
//
//     /// The height of this dimension.
//     pub(crate) fn height(&self) -> u32 {
//         self.height
//     }
//
//     /// The depth of this dimension.
//     pub(crate) fn depth(&self) -> u32 {
//         self.depth
//     }
//
//     /// The maximum X value of this dimension.
//     pub(crate) fn x_max(&self) -> u32 {
//         self.width - 1
//     }
//
//     /// The maximum Y value of this dimension.
//     pub(crate) fn y_max(&self) -> u32 {
//         self.height - 1
//     }
//
//     /// The maximum Z value of this dimension.
//     pub(crate) fn z_max(&self) -> u32 {
//         self.depth - 1
//     }
//
//     /// Returns the center as a `Vec3`.
//     pub(crate) fn center(&self) -> Point3 {
//         Point3::new(
//             self.width as i32 / 2,
//             self.height as i32 / 2,
//             self.depth as i32 / 2,
//         )
//     }
//
//     /// Checks if a given coordinate is within bounds of the `Chunk`.
//     pub(crate) fn check_point(&self, point: Point3) -> DimensionResult<()> {
//         if point.x() > self.width as i32
//             || point.y() > self.height as i32
//             || point.z() > self.depth as i32
//         {
//             Err(ErrorKind::OutOfBounds.into())
//         } else {
//             Ok(())
//         }
//     }
//
//     /// Checks if a given index is within bounds of the `Chunk`.
//     pub(crate) fn check_index(&self, index: usize) -> DimensionResult<()> {
//         if index > (self.width * self.height * self.depth) as usize {
//             Err(ErrorKind::OutOfBounds.into())
//         } else {
//             Ok(())
//         }
//     }
//
//     /// Encodes a Vec3 coordinate to an usize index to use in the Tile vector, unchecked.
//     pub(crate) fn encode_point_unchecked(&self, point: Point3) -> usize {
//         ((point.z() * self.width as i32 * self.height as i32)
//             + (point.y() * self.width as i32)
//             + point.x()) as usize
//     }
//
//     /// Encodes a `Vec3` coordinate to an `usize` index to use in the `Tile` vector.
//     pub(crate) fn encode_point(&self, point: Point3) -> DimensionResult<usize> {
//         self.check_point(point)?;
//         Ok(self.encode_point_unchecked(point))
//     }
//
//     /// Decodes a Tile index and returns the coordinates in the Chunk, unchecked.
//     pub(crate) fn decode_coord_unchecked(&self, index: usize) -> Point3 {
//         let z = index as u32 / (self.width * self.height);
//         let index = index as u32 - (z * self.width * self.height);
//         let y = self.y_max() - (index / self.width);
//         let x = index % self.width;
//         Point3::new(x as i32, y as i32, z as i32)
//     }
//
//     /// Decodes a `Tile` index and returns the coordinates in the `Chunk`.
//     pub(crate) fn decode_coord(&self, index: usize) -> DimensionResult<Point3> {
//         self.check_index(index)?;
//         Ok(self.decode_coord_unchecked(index))
//     }
// }
//
// impl From<Dimension3> for (u32, u32, u32) {
//     fn from(dimensions: Dimension3) -> (u32, u32, u32) {
//         (dimensions.width(), dimensions.height(), dimensions.depth())
//     }
// }
