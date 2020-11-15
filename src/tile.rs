use crate::lib::*;

/// A tile with an index value and color.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Tile {
    index: u32,
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
    pub fn new(index: u32) -> Tile {
        Tile {
            index,
            ..Default::default()
        }
    }

    /// Creates a new `Tile` with a color and a given index.
    pub fn with_color(index: u32, color: Color) -> Tile {
        Tile { index, color }
    }

    /// Returns the sprite index value.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Returns the coordinate in the texture if any.
    pub fn color(&self) -> Color {
        self.color
    }
}

/// A tool used to set multiple tiles at a time.
///
/// This is the preferred and fastest way to set tiles. Optionally, you can set them individually.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TileSetter(Vec<(Vec3, Tile)>);

impl TileSetter {
    /// Returns a new `TileSetter` with the type `Tile`.
    pub fn new() -> TileSetter {
        TileSetter(Vec::new())
    }

    /// Returns a new `TileSetter` with a specified capacity with the type `Tile`.
    pub fn with_capacity(capacity: usize) -> TileSetter {
        TileSetter(Vec::with_capacity(capacity))
    }

    /// Pushes a single tile with a coordinate into the `TileSetter`.
    pub fn push(&mut self, coord: Vec3, tile: Tile) {
        self.0.push((coord, tile));
    }

    // /// Pushes a stack of tiles to be rendered from background to foreground.
    // pub fn push_stack(&mut self, coord: Vec3, tile: Tile) {
    //     self.0.push((coord, tile))
    // }

    /// Returns the length of the `TileSetter`.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns if the `TileSetter` is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates over all coordinates and tiles in the `TileSetter`.
    pub fn iter(&self) -> Iter<'_, (Vec3, Tile)> {
        self.0.iter()
    }

    /// Mutably iterates over all coordinates and tiles in the `TileSetter`.
    pub fn iter_mut(&mut self) -> IterMut<'_, (Vec3, Tile)> {
        self.0.iter_mut()
    }
}
