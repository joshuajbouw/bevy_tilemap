//! Tile traits to implement for a custom tile.

use crate::lib::*;

/// A tile with an index value and color.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Tile<P: Into<Point3>> {
    /// A point where the tile will exist.
    pub point: P,
    /// The Z order layer of the tile. Higher will place the tile above others.
    pub sprite_order: usize,
    /// The sprites index in the texture atlas.
    pub sprite_index: usize,
    /// The desired tint and alpha of the tile. White means no change.
    pub tint: Color,
}

impl<P: Into<Point3> + Default> Default for Tile<P> {
    fn default() -> Tile<P> {
        Tile {
            point: P::default(),
            sprite_order: 0,
            sprite_index: 0,
            tint: Color::WHITE,
        }
    }
}
