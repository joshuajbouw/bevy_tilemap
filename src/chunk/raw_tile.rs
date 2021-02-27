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
    dimension: Dimension3,
    tiles: &HashMap<usize, RawTile>,
) -> (Vec<f32>, Vec<[f32; 4]>) {
    let area = (dimension.width * dimension.height) as usize;
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
