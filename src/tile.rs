use crate::{lib::*, point::Point3};

/// A hash map type to use for setting tiles.
pub type Tiles = HashMap<(i32, i32, i32), Tile>;

pub(crate) type TilePoints = HashMap<Point3, Tile>;

// TODO: Fix both these renderer parts below to only include the current depth.
/// A utility function that takes an array of `Tile`s and splits the indexes and
/// colors and returns them as separate vectors for use in the renderer.
pub(crate) fn dense_tiles_to_attributes(tiles: &[Tile]) -> (Vec<f32>, Vec<[f32; 4]>) {
    let capacity = tiles.len() * 4;
    let mut tile_indexes: Vec<f32> = Vec::with_capacity(capacity);
    let mut tile_colors: Vec<[f32; 4]> = Vec::with_capacity(capacity);
    for tile in tiles.iter() {
        tile_indexes.extend([tile.index() as f32; 4].iter());
        tile_colors.extend([tile.color().into(); 4].iter());
    }
    (tile_indexes, tile_colors)
}

/// A utility function that takes a sparse map of `Tile`s and splits the indexes
/// and colors and returns them as separate vectors for use in the renderer.
pub(crate) fn sparse_tiles_to_attributes(
    area: usize,
    tiles: &HashMap<usize, Tile>,
) -> (Vec<f32>, Vec<[f32; 4]>) {
    let mut tile_indexes = vec![0.; area * 4];
    // If tiles are set with an alpha of 0, they are discarded.
    let mut tile_colors = vec![[0.0, 0.0, 0.0, 0.0]; area * 4];
    for (index, tile) in tiles.iter() {
        for i in 0..4 {
            tile_indexes[index * 4 + i] = tile.index as f32;
            tile_colors[index * 4 + i] = tile.color.into();
        }
    }
    (tile_indexes, tile_colors)
}

/// A tile with an index value and color.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Tile {
    index: usize,
    color: Color,
}

impl Default for Tile {
    fn default() -> Tile {
        Tile {
            index: 0,
            color: Color::WHITE,
        }
    }
}

impl Tile {
    /// Creates a new `Tile` with a given index.
    ///
    /// By default, this makes a tile with no tint to the color at all. If tile
    /// tinting is needed, use `with_color` instead.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    ///
    /// // Creates a tile with an index of 0
    /// let tile = Tile::new(0);
    /// ```
    pub fn new(index: usize) -> Tile {
        Tile {
            index,
            ..Default::default()
        }
    }

    /// Creates a new `Tile` with a color and a given index.
    ///
    /// A color is handy if you want full tinting done on a tile.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    /// use bevy::prelude::*;
    ///
    /// let tile = Tile::with_color(0, Color::BLUE);
    /// ```
    pub fn with_color(index: usize, color: Color) -> Tile {
        Tile { index, color }
    }

    /// Returns the sprite index value.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    ///
    /// let index = 0;
    /// let tile = Tile::new(0);
    ///
    /// assert_eq!(index, tile.index());
    /// ```
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the tint color of the tile.
    ///
    /// Most cases this is white which simply means that no tint has been
    /// applied.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    /// use bevy::prelude::*;
    ///
    /// let color = Color::GREEN;
    /// let tile = Tile::with_color(0, color);
    ///
    /// assert_eq!(color, tile.color());
    /// ```
    pub fn color(&self) -> Color {
        self.color
    }
}
