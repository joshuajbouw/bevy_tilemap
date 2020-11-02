use crate::lib::*;

pub trait Tile: 'static + Debug + Default + Clone + Send + Sync {
    /// The constant width in pixels.
    const WIDTH: f32;
    /// The constant height in pixels.
    const HEIGHT: f32;

    /// Returns the sprite index value.
    fn texture(&self) -> Option<&Handle<Texture>>;

    fn coord(&self) -> Option<Vec2>;
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TileSetter<T: Tile>(Vec<(Vec3, T)>);

impl<T: Tile> TileSetter<T> {
    pub fn new() -> TileSetter<T> {
        TileSetter(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> TileSetter<T> {
        TileSetter(Vec::with_capacity(capacity))
    }

    pub fn push(&mut self, coord: Vec3, tile: T) {
        self.0.push((coord, tile));
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, (Vec3, T)> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, (Vec3, T)> {
        self.0.iter_mut()
    }
}