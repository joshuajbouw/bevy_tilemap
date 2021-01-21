use crate::lib::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Debug)]
/// A raw tile composed of simply an index and a color.
pub struct RawTile {
    /// The index of the tile in the sprite sheet.
    pub index: usize,
    /// The color, or tint, of the tile.
    pub color: Color,
}

impl Default for RawTile {
    fn default() -> Self {
        RawTile {
            index: 0,
            color: Color::WHITE,
        }
    }
}

/// A tile with an index value and color.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Debug)]
#[non_exhaustive]
pub struct Tile {
    /// A point where the tile will exist.
    pub point: Point2,
    /// The Z order layer of the tile. Higher will place the tile above others.
    pub z_order: usize,
    /// The sprites index in the texture atlas.
    pub sprite_index: usize,
    /// The desired tint and alpha of the tile. White means no change.
    pub tint: Color,
}

impl Default for Tile {
    fn default() -> Tile {
        Tile {
            point: Point2::new(0, 0),
            z_order: 0,
            sprite_index: 0,
            tint: Color::WHITE,
        }
    }
}

// TODO: Remove this and replace with builder.
impl Tile {
    /// Creates a new tile with a provided point and tile index.
    ///
    /// By default, this makes a tile with no tint to the color at all. If tile
    /// tinting is needed, use [`with_tint`] instead.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    ///
    /// // Creates a tile with an index of 0 at point 3x,3y
    /// let tile = Tile::new((3, 3), 0);
    /// ```
    ///
    /// [`Tile`]: Tile
    /// [`with_tint`]: Tile::with_tint
    pub fn new<P: Into<Point2>>(point: P, sprite_index: usize) -> Tile {
        Tile {
            point: point.into(),
            z_order: 0,
            sprite_index,
            tint: Color::WHITE,
        }
    }

    /// Creates a new tile with a given Z order and sprite index at a point.
    pub fn with_z_order<P: Into<Point2>>(point: P, sprite_index: usize, z_order: usize) -> Tile {
        Tile {
            point: point.into(),
            z_order,
            sprite_index,
            tint: Color::WHITE,
        }
    }

    /// Creates a new tile with a color and a given sprite index.
    ///
    /// The color argument implements `Into<[`Color`]>`.
    ///
    /// # Examples
    /// ```
    /// use bevy_render::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// let point = (15, 15);
    /// let sprite_index = 3;
    /// let tint = Color::BLUE;
    ///
    /// let tile = Tile::with_tint(point, sprite_index, tint);
    /// ```
    ///
    /// [`Color`]: Bevy::render::color::Color
    pub fn with_tint<P: Into<Point2>, C: Into<Color>>(
        point: P,
        sprite_index: usize,
        tint: C,
    ) -> Tile {
        Tile {
            point: point.into(),
            z_order: 0,
            sprite_index,
            tint: tint.into(),
        }
    }

    /// Crates a new tile with a given color, index and color at a point.
    ///
    /// The color argument implements `Into<[`Color`]>`.
    ///
    /// # Examples
    /// ```
    /// use bevy_render::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// let point = (15, 15);
    /// let z_order = 0;
    /// let sprite_index = 2;
    /// let tint = Color::RED;
    ///
    /// let tile = Tile::with_z_order_and_tint(point, z_order, sprite_index, tint);
    /// ```
    pub fn with_z_order_and_tint<P: Into<Point2>, C: Into<Color>>(
        point: P,
        sprite_index: usize,
        z_order: usize,
        tint: C,
    ) -> Tile {
        Tile {
            point: point.into(),
            z_order,
            sprite_index,
            tint: tint.into(),
        }
    }
}

/// A utility function that takes an array of `Tile`s and splits the indexes and
/// colors and returns them as separate vectors for use in the renderer.
pub(crate) fn dense_tiles_to_attributes(tiles: &[RawTile]) -> (Vec<f32>, Vec<[f32; 4]>) {
    let capacity = tiles.len() * 4;
    let mut tile_indexes: Vec<f32> = Vec::with_capacity(capacity);
    let mut tile_colors: Vec<[f32; 4]> = Vec::with_capacity(capacity);
    for tile in tiles.iter() {
        tile_indexes.extend([tile.index as f32; 4].iter());
        tile_colors.extend([tile.color.into(); 4].iter());
    }
    (tile_indexes, tile_colors)
}

/// A utility function that takes a sparse map of `Tile`s and splits the indexes
/// and colors and returns them as separate vectors for use in the renderer.
pub(crate) fn sparse_tiles_to_attributes(
    area: usize,
    tiles: &HashMap<usize, RawTile>,
) -> (Vec<f32>, Vec<[f32; 4]>) {
    let mut tile_indexes = vec![0.; area * 4];
    // If tiles are set with an alpha of 0, they are discarded.
    let mut tile_colors = vec![[0.0, 0.0, 0.0, 0.0]; area * 4];
    for (index, tile) in tiles.iter() {
        for i in 0..4 {
            if let Some(index) = tile_indexes.get_mut(index * 4 + i) {
                *index = tile.index as f32;
            }
            if let Some(index) = tile_colors.get_mut(index * 4 + i) {
                *index = tile.color.into();
            }
        }
    }
    (tile_indexes, tile_colors)
}
