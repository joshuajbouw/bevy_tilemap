use crate::{
    coord::ToWorldCoordinates,
    dimensions::{DimensionResult, Dimensions3},
    lib::*,
    map::TileMap,
    tile::Tile,
};

/// A **chunk** that can be used in a `TileMap`.
///
/// Provides standard `Chunk` implementations to be used in the library's
/// systems and methods.
pub trait Chunk<T: Tile>: 'static + Dimensions3 + TypeUuid + Default + Send + Sync {
    /// The constant width in `Tile`s.
    const WIDTH: f32;
    /// The constant height in `Tile`s.
    const HEIGHT: f32;
    /// The constant depth in `Tile`s.
    const DEPTH: f32;

    /// The constant maximum X value in `Tile`s.
    const X_MAX: f32 = Self::WIDTH - 1.;

    /// The constant maximum Y value in `Tile`s.
    const Y_MAX: f32 = Self::HEIGHT - 1.;

    /// The constant maximum Z value in `Tile`s.
    const Z_MAX: f32 = Self::DEPTH - 1.;

    /// Sets the texture handle with a new one.
    ///
    /// # Warning
    /// This should **only** be used when creating a new `Chunk` from the
    /// `default` method.
    fn set_texture_handle(&mut self, handle: Option<Handle<Texture>>);

    /// Returns a copy of the dimensions in `Tile`s.
    fn tile_dimensions(&self) -> Vec2;

    /// Returns a copy of the pixel dimensions.
    fn pixel_dimensions(&self) -> Vec2;

    /// Returns a reference to the `Texture` `Handle`.
    fn texture_handle(&self) -> Option<&Handle<Texture>>;

    /// Returns a reference to the vector of the texture `Rect` coordinates.
    fn textures(&self) -> &[Rect];

    /// Returns a reference to the `Tile` in the `Chunk`, if it exists.
    fn tile(&self, coord: &Vec3) -> DimensionResult<Option<&T>>;

    /// Returns a reference to the `Tile` vector.
    fn tiles(&self) -> &Vec<Option<T>>;

    /// Cleans all the unneeded parameters when despawning.
    fn clean(&mut self);
}

/// A basic use of the `Chunk` trait that has the bare minimum methods.
///
/// Serde skips the textures and texture_handle field for three reasons:
/// * Handle doesn't support it.
/// * Rect doesn't support it.
/// * Even if the above supported it, there shouldn't be a need to store that
/// information anyways as they are temporary.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorldChunk<T: Tile> {
    #[serde(skip)]
    /// The texture handle that gets ultimately rendered.
    texture_handle: Option<Handle<Texture>>,
    #[serde(skip)]
    /// A vector of `Rect`s that share where the sprite is located.
    textures: Vec<Rect>,
    /// A vector of all the tiles in the `TileMap`.
    tiles: Vec<Option<T>>,
}

impl<T: Tile> TypeUuid for WorldChunk<T> {
    const TYPE_UUID: Uuid = Uuid::from_u128(45182109655678555067446040298151572788);
}

impl<T: Tile> Default for WorldChunk<T> {
    fn default() -> WorldChunk<T> {
        let mut textures = Vec::new();
        for y in 0..Self::WIDTH as u32 {
            for x in 0..Self::HEIGHT as u32 {
                textures.push(Rect {
                    min: Vec2::new(x as f32 * T::WIDTH, y as f32 * T::HEIGHT),
                    max: Vec2::new((x + 1) as f32 * T::WIDTH, (y + 1) as f32 * T::HEIGHT),
                })
            }
        }
        WorldChunk {
            texture_handle: Default::default(),
            textures,
            tiles: Vec::new(),
        }
    }
}

impl<T: Tile> Dimensions3 for WorldChunk<T> {
    fn dimensions(&self) -> Vec3 {
        Vec3::new(Self::WIDTH, Self::HEIGHT, Self::DEPTH)
    }
}

impl<T: Tile> Chunk<T> for WorldChunk<T> {
    const WIDTH: f32 = 32.0;
    const HEIGHT: f32 = 32.0;
    const DEPTH: f32 = 512.0;

    fn set_texture_handle(&mut self, handle: Option<Handle<Texture>>) {
        self.texture_handle = handle;
    }

    fn tile_dimensions(&self) -> Vec2 {
        Vec2::new(T::WIDTH, T::HEIGHT)
    }

    fn pixel_dimensions(&self) -> Vec2 {
        Vec2::new(T::WIDTH * Self::WIDTH, T::HEIGHT * Self::HEIGHT)
    }

    fn texture_handle(&self) -> Option<&Handle<Texture>> {
        self.texture_handle.as_ref()
    }

    fn textures(&self) -> &[Rect] {
        &self.textures
    }

    fn tile(&self, coord: &Vec3) -> DimensionResult<Option<&T>> {
        let idx = self.encode_coord(coord)?;
        Ok(self.tiles[idx].as_ref())
    }

    fn tiles(&self) -> &Vec<Option<T>> {
        &self.tiles
    }

    fn clean(&mut self) {
        self.texture_handle = None;
        self.textures.clear();
        self.textures.shrink_to_fit();
    }
}

impl<T: Tile> Drop for WorldChunk<T> {
    fn drop(&mut self) {
        println!("World chunk de-spawned");
        self.clean();
    }
}

impl<T: Tile, M: TileMap<T, Self>> ToWorldCoordinates<T, Self, M> for WorldChunk<T> {}

impl<T: Tile> WorldChunk<T> {
    /// Returns a new `WorldChunk`.
    ///
    /// # Arguments
    /// * texture_handle - Takes in a `Handle<Texture>` to store for use with
    /// getting the correct texture from Bevy assets.
    pub fn new(texture_handle: Handle<Texture>) -> WorldChunk<T> {
        let mut sprites = Vec::new();
        for y in 0..Self::WIDTH as u32 {
            for x in 0..Self::HEIGHT as u32 {
                sprites.push(Rect {
                    min: Vec2::new(x as f32 * T::WIDTH, y as f32 * T::HEIGHT),
                    max: Vec2::new((x + 1) as f32 * T::WIDTH, (y + 1) as f32 * T::HEIGHT),
                })
            }
        }
        WorldChunk {
            texture_handle: Some(texture_handle),
            textures: sprites,
            tiles: vec![None; (Self::WIDTH * Self::HEIGHT) as usize],
        }
    }
}
