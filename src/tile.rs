//! Tile traits to implement for a custom tile.

use crate::lib::*;

/// A tile with an index value and color.

#[derive(Component, Copy, Clone, PartialEq, Debug)]
pub struct Tile<P: Into<Point3> + Sync + Send + 'static> {
    /// A point where the tile will exist.
    pub point: P,
    /// The Z order layer of the tile. Higher will place the tile above others.
    pub sprite_order: usize,
    /// The sprites index in the texture atlas.
    pub sprite_index: usize,
    /// The desired tint and alpha of the tile. White means no change.
    pub tint: Color,
    pub flip_x: bool,
    pub flip_y: bool,
    pub flip_d: bool,
    pub visible: bool,
}

impl<P: Into<Point3> + Sync + Send + 'static> TypeUuid for Tile<P> {
    const TYPE_UUID: Uuid = Uuid::from_u128(120662740761204452827358747683215577593);
}

impl<P: Into<Point3> + Default + Sync + Send> Default for Tile<P> {
    fn default() -> Tile<P> {
        Tile {
            point: P::default(),
            sprite_order: 0,
            sprite_index: 0,
            tint: Color::WHITE,
            flip_x: false,
            flip_y: false,
            flip_d: false,
            visible: false,
        }
    }
}
