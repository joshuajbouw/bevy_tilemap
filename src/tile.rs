use crate::lib::*;

/// A utility function that takes an array of `Tile`s and splits the indexes and
/// colors and returns them as separate vectors for use in the renderer.
pub fn tiles_to_renderer_parts(tiles: &[Tile]) -> (Vec<f32>, Vec<[f32; 4]>) {
    let mut tile_indexes: Vec<f32> = Vec::with_capacity(tiles.len());
    let mut tile_colors: Vec<[f32; 4]> = Vec::with_capacity(tiles.len());
    for tile in tiles.iter() {
        tile_indexes.push(tile.index() as f32);
        tile_colors.push(tile.color().into());
    }
    (tile_indexes, tile_colors)
}

/// A tile with an index value and color.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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
    /// use bevy_tilemap::Tile;
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
    /// use bevy_tilemap::Tile;
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
    /// use bevy_tilemap::Tile;
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
    /// use bevy_tilemap::Tile;
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

/// A tool used to set multiple tiles at a time.
///
/// This is the preferred and fastest way to set tiles. Optionally, you can set them individually.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TileSetter(Vec<(Vec3, Tile)>);

impl TileSetter {
    /// Constructs a new `TileSetter` with the type `Tile`.
    ///
    /// This is effectively a wrapped vector with a tuple of a coordinate and
    /// `Tile`. By itself it does not set the tiles but it is used as a helper
    /// to keep data clear and concise.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::TileSetter;
    ///
    /// let mut setter = TileSetter::new();
    /// ```
    pub fn new() -> TileSetter {
        TileSetter(Vec::new())
    }

    /// Constructs a new `TileSetter` with a specified capacity with the type `Tile`.
    ///
    /// The vector will be able to hold exactly `capacity` elements without
    /// reallocating. If `capacity` is 0, the vector will not allocate.
    ///
    /// It is important to note that although the returned vector has the
    /// *capacity* specified, the vector will have a zero *length*.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::{TileSetter, Tile};
    /// use bevy::prelude::*;
    ///
    /// let mut setter = TileSetter::with_capacity(10);
    ///
    /// // The setter contains no items, even though it has capacity for more
    /// assert_eq!(setter.len(), 0);
    /// assert_eq!(setter.capacity(), 10);
    ///
    /// // These are all done without reallocating...
    /// for i in 0..10 {
    ///     let coord = Vec3::new(i as f32, i as f32 + 1., 0.);
    ///     let tile = Tile::new(i);
    ///     setter.push(coord, tile);
    /// }
    /// assert_eq!(setter.len(), 10);
    /// assert_eq!(setter.capacity(), 10);
    ///
    /// // ...but this may make the vector reallocate
    /// let coord = Vec3::new(11., 12., 0.);
    /// let tile = Tile::new(11);
    /// setter.push(coord, tile);
    /// assert_eq!(setter.len(), 11);
    /// assert!(setter.capacity() >= 11);
    /// ```
    pub fn with_capacity(capacity: usize) -> TileSetter {
        TileSetter(Vec::with_capacity(capacity))
    }

    /// Pushes a single tile with a coordinate into the `TileSetter`.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes;
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::{TileSetter, Tile};
    /// use bevy::prelude::*;
    ///
    /// let mut setter = TileSetter::new();
    /// let coord = Vec3::new(1., 1., 0.);
    /// let tile = Tile::new(1);
    /// setter.push(coord, tile);
    /// ```
    pub fn push(&mut self, coord: Vec3, tile: Tile) {
        self.0.push((coord, tile));
    }

    // /// Pushes a stack of tiles to be rendered from background to foreground.
    // pub fn push_stack(&mut self, coord: Vec3, tile: Tile) {
    //     self.0.push((coord, tile))
    // }

    /// Shrinks the capacity of the `TileSetter` as much as possible.
    ///
    /// It will drop down as much as possible to the length but the allocator
    /// may still inform the vector that there is space for a few more elements.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::{TileSetter, Tile};
    /// use bevy::prelude::*;
    ///
    /// let mut setter = TileSetter::with_capacity(10);
    /// let coord = Vec3::new(1., 1., 0.);
    /// let tile = Tile::new(1);
    /// setter.push(coord, tile);
    /// assert_eq!(setter.capacity(), 10);
    /// setter.shrink_to_fit();
    /// assert!(setter.capacity() >= 1);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    /// Reserves the minimum capacity for exactly additional more elements to be
    /// inserted in the given `TileSetter`.
    ///
    /// After calling `reserve_exact`, capacity will be greater than or equal to
    /// self.len() + additional. Does nothing if the capacity is already
    /// sufficient.
    ///
    /// Note that the allocator may give the collection more space than it
    /// requests. Therefore, capacity can not be relied upon to be precisely
    /// normal. Prefer `reserve` if future insertions are expected.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::TileSetter;
    ///
    /// let mut setter = TileSetter::new();
    /// setter.reserve_exact(10);
    /// assert!(setter.capacity() >= 10);
    /// ```
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional);
    }

    /// Returns the number of elements in the `TileSetter`, also referred to as
    /// its 'length'.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::TileSetter;
    ///
    /// let mut setter = TileSetter::new();
    /// assert_eq!(setter.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the number of elements in the vector can hold without
    /// reallocating.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::TileSetter;
    ///
    /// let mut setter = TileSetter::with_capacity(10);
    /// assert_eq!(setter.capacity(), 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Returns if the `TileSetter` is empty and contains no elements.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::{TileSetter, Tile};
    /// use bevy::prelude::*;
    ///
    /// let mut setter = TileSetter::new();
    /// assert!(setter.is_empty());
    ///
    /// let coord = Vec3::new(1., 1., 0.);
    /// let tile = Tile::new(1);
    /// setter.push(coord, tile);
    /// assert!(!setter.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over all coordinates and tiles in the `TileSetter`.
    pub fn iter(&self) -> Iter<'_, (Vec3, Tile)> {
        self.0.iter()
    }

    /// Returns a mutable iterator over all coordinates and tiles in the `TileSetter`.
    pub fn iter_mut(&mut self) -> IterMut<'_, (Vec3, Tile)> {
        self.0.iter_mut()
    }
}

impl Extend<(Vec3, Tile)> for TileSetter {
    fn extend<I: IntoIterator<Item = (Vec3, Tile)>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl From<Vec<(Vec3, Tile)>> for TileSetter {
    fn from(vec: Vec<(Vec3, Tile)>) -> TileSetter {
        TileSetter(vec)
    }
}
