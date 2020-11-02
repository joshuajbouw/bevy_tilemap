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
