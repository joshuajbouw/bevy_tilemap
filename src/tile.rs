use crate::lib::*;

/// A generic `Tile` trait to be used in a `Chunk`.
pub trait Tile: 'static + Debug + Default + Clone + Send + Sync {
    /// The constant width in pixels.
    const WIDTH: f32;
    /// The constant height in pixels.
    const HEIGHT: f32;

    /// Returns the sprite index value.
    fn texture(&self) -> Option<&Handle<Texture>>;

    /// Returns the coordinate in the texture if any.
    fn coord(&self) -> Option<Vec2>;
}

/// A tool used to set multiple tiles at a time.
///
/// This is the preferred and fastest way to set tiles. Optionally, you can set them individually.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TileSetter<T: Tile>(Vec<(Vec3, Vec<T>)>);

impl<T: Tile> TileSetter<T> {
    /// Returns a new `TileSetter` with the type `Tile`.
    pub fn new() -> TileSetter<T> {
        TileSetter(Vec::new())
    }

    /// Returns a new `TileSetter` with a specified capacity with the type `Tile`.
    pub fn with_capacity(capacity: usize) -> TileSetter<T> {
        TileSetter(Vec::with_capacity(capacity))
    }

    /// Pushes a single tile with a coordinate into the `TileSetter`.
    pub fn push(&mut self, coord: Vec3, tile: T) {
        self.0.push((coord, vec![tile]));
    }

    /// Pushes a stack of tiles to be rendered from background to foreground.
    pub fn push_stack(&mut self, coord: Vec3, tiles: Vec<T>) {
        self.0.push((coord, tiles))
    }

    /// Returns the length of the `TileSetter`.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns if the `TileSetter` is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates over all coordinates and tiles in the `TileSetter`.
    pub fn iter(&self) -> Iter<'_, (Vec3, Vec<T>)> {
        self.0.iter()
    }

    /// Mutably iterates over all coordinates and tiles in the `TileSetter`.
    pub fn iter_mut(&mut self) -> IterMut<'_, (Vec3, Vec<T>)> {
        self.0.iter_mut()
    }
}
