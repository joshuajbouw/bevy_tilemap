use crate::lib::*;

#[derive(PartialEq)]
enum ErrorKind {
    DimensionError(DimensionError),
    RectanglePackError(RectanglePackError),
    NotEnoughSpace,
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use ErrorKind::*;
        match self {
            DimensionError(err) => ::std::fmt::Debug::fmt(&err, f),
            RectanglePackError(err) => ::std::fmt::Debug::fmt(&err, f),
            NotEnoughSpace => write!(
                f,
                "not enough space in the sprite sheet, increase the maximum size"
            ),
        }
    }
}

#[derive(PartialEq, Debug)]
/// The error type for operations when interacting with a sprite sheet.
pub struct SpriteSheetError(Box<ErrorKind>);

impl Display for SpriteSheetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl Error for SpriteSheetError {}

impl From<DimensionError> for SpriteSheetError {
    fn from(err: DimensionError) -> SpriteSheetError {
        SpriteSheetError(Box::new(ErrorKind::DimensionError(err)))
    }
}

impl From<ErrorKind> for SpriteSheetError {
    fn from(kind: ErrorKind) -> SpriteSheetError {
        SpriteSheetError(Box::new(kind))
    }
}

impl From<RectanglePackError> for SpriteSheetError {
    fn from(err: RectanglePackError) -> SpriteSheetError {
        SpriteSheetError(Box::new(ErrorKind::RectanglePackError(err)))
    }
}

/// A sprite sheet result.
pub type SpriteSheetResult<T> = Result<T, SpriteSheetError>;

/// A builder which is used to create a sprite sheet.
pub struct SpriteSheetBuilder {
    rects_to_place: GroupedRectsToPlace<(Handle<Texture>, usize)>,
    sprite_size: Dimension2,
    max_size: Dimension2,
}

impl Default for SpriteSheetBuilder {
    fn default() -> SpriteSheetBuilder {
        SpriteSheetBuilder {
            rects_to_place: GroupedRectsToPlace::new(),
            sprite_size: Dimension2::new(64, 64),
            max_size: Dimension2::new(2048, 2048),
        }
    }
}

impl SpriteSheetBuilder {
    /// Constructs a new sprite sheet builder then is consumed to create a new
    /// sprite sheet.
    pub fn new() -> SpriteSheetBuilder {
        SpriteSheetBuilder::default()
    }

    /// Sets the dimensions per sprite in pixels.
    pub fn sprite_dimensions<D: Into<Dimension2>>(mut self, sprite_size: D) -> SpriteSheetBuilder {
        self.sprite_size = sprite_size.into();
        self
    }

    /// Sets the maximum dimensions of the sprite sheet in pixels.
    pub fn max_dimensions<D: Into<Dimension2>>(mut self, max_size: D) -> SpriteSheetBuilder {
        self.max_size = max_size.into();
        self
    }

    /// Adds a sprite to the sprite sheet texture.
    pub fn add_sprite<D: Into<Dimension2>>(
        mut self,
        texture_handle: Handle<Texture>,
        texture_dimensions: D,
    ) -> SpriteSheetBuilder {
        let texture_dimensions = texture_dimensions.into();
        self.rects_to_place.push_rect(
            (texture_handle, 0),
            None,
            RectToInsert::new(texture_dimensions.width, texture_dimensions.height, 1),
        );
        self
    }

    /// Adds multiple sprites already in a single texture or, can be used to
    /// combine multiple sprite sheets.
    pub fn add_sprites<D: Into<Dimension2>>(
        &mut self,
        texture_handle: Handle<Texture>,
        texture_dimensions: D,
        cell_size: D,
    ) {
        let texture_dimensions = texture_dimensions.into();
        let cell_size = cell_size.into();
        let width = texture_dimensions.width / cell_size.width;
        let height = texture_dimensions.height as u32 / cell_size.height;

        for y in 0..height {
            for x in 0..width {
                let index = ((y * width) + x) as usize;
                self.rects_to_place.push_rect(
                    (texture_handle.clone_weak(), index),
                    None,
                    RectToInsert::new(width * cell_size.width, height * cell_size.height, 1),
                )
            }
        }
    }

    fn place_texture(
        &mut self,
        atlas_texture: &mut Texture,
        texture: &Texture,
        packed_location: &PackedLocation,
    ) {
        let rect_width = packed_location.width() as usize;
        let rect_height = packed_location.height() as usize;
        let rect_x = packed_location.x() as usize;
        let rect_y = packed_location.y() as usize;
        let atlas_width = atlas_texture.size.x() as usize;
        let format_size = atlas_texture.format.pixel_size();

        for (texture_y, bound_y) in (rect_y..rect_y + rect_height).enumerate() {
            let begin = (bound_y * atlas_width + rect_x) * format_size;
            let end = begin + rect_width * format_size;
            let texture_begin = texture_y * rect_width * format_size;
            let texture_end = texture_begin + rect_width * format_size;
            atlas_texture.data[begin..end]
                .copy_from_slice(&texture.data[texture_begin..texture_end]);
        }
    }

    /// Consumes the builder, runs some final operations, and returns a new
    /// sprite sheet with a single texture.
    pub fn finish(mut self, textures: &mut Assets<Texture>) -> SpriteSheetResult<SpriteSheet> {
        let initial_width = self.sprite_size.width;
        let initial_height = self.sprite_size.height;
        let max_width = self.max_size.width;
        let max_height = self.max_size.height;

        let mut current_width = initial_width;
        let mut current_height = initial_height;
        let mut rect_placements = None;
        let mut atlas_texture = Texture::default();

        while rect_placements.is_none() {
            if current_width > max_width || current_height > max_height {
                rect_placements = None;
                break;
            }
            let mut target_bins = BTreeMap::new();
            target_bins.insert(0, TargetBin::new(current_width, current_height, 1));
            atlas_texture = Texture::new_fill(
                Vec2::new(current_width as f32, current_height as f32),
                &[0, 0, 0, 0],
                TextureFormat::Rgba8UnormSrgb,
            );
            rect_placements = match pack_rects(
                &self.rects_to_place,
                target_bins,
                &volume_heuristic,
                &contains_smallest_box,
            ) {
                Ok(rect_placements) => Some(rect_placements),
                Err(RectanglePackError::NotEnoughBinSpace) => {
                    current_width *= 2;
                    current_height *= 2;
                    None
                }
            }
        }

        let rect_placements =
            rect_placements.ok_or_else(|| SpriteSheetError::from(ErrorKind::NotEnoughSpace))?;

        let mut texture_rects = Vec::with_capacity(rect_placements.packed_locations().len());
        let mut texture_handles = HashMap::default();
        for ((texture_handle, index), (_, packed_location)) in
            rect_placements.packed_locations().iter()
        {
            let texture = textures.get(texture_handle).unwrap();
            let min = Vec2::new(packed_location.x() as f32, packed_location.y() as f32);
            let max = min
                + Vec2::new(
                    packed_location.width() as f32,
                    packed_location.height() as f32,
                );
            texture_handles
                .entry(texture_handle.clone_weak())
                .or_insert({
                    let mut indices = HashMap::default();
                    indices.insert(*index, texture_rects.len());
                    indices
                });
            texture_rects.push(Rect { min, max });
            self.place_texture(&mut atlas_texture, texture, packed_location);
        }
        Ok(SpriteSheet {
            size: atlas_texture.size.into(),
            dimensions: Dimension2::new(
                atlas_texture.size.x() as u32 / self.sprite_size.width,
                atlas_texture.size.y() as u32 / self.sprite_size.height,
            ),
            texture: textures.add(atlas_texture),
            sprites: texture_rects,
            sprite_handles: Some(texture_handles),
        })
    }
}

#[derive(Debug, RenderResources, TypeUuid)]
#[uuid = "0ae39dfc-2d54-4c2f-bbe0-29a41d6518b5"]
/// A sprite sheet which is used to get individual sprites from a single
/// texture.
pub struct SpriteSheet {
    texture: Handle<Texture>,
    #[render_resources(ignore)]
    dimensions: Dimension2,
    #[render_resources(ignore)]
    size: Dimension2,
    #[render_resources(buffer)]
    sprites: Vec<Rect>,
    #[render_resources(ignore)]
    sprite_handles: Option<HashMap<Handle<Texture>, HashMap<usize, usize>>>,
}

impl SpriteSheet {
    /// Constructs a new sprite sheet with a single texture that has padding.
    pub fn with_padding<D: Into<Dimension2>>(
        texture: Handle<Texture>,
        tile_dimensions: D,
        columns: u32,
        rows: u32,
        padding: D,
    ) -> SpriteSheet {
        let tile_dimensions = tile_dimensions.into();
        let padding = padding.into();

        let mut sprites: Vec<Rect> = Vec::with_capacity((columns * rows) as usize);

        assert!(padding.width > 0);
        assert!(padding.height > 0);

        let mut x_padding = 0;
        let mut y_padding = 0;

        for y in 0..rows {
            if y > 0 {
                y_padding = padding.width;
            }
            for x in 0..columns {
                if x > 0 {
                    x_padding = padding.height;
                }

                let rect_min = Vec2::new(
                    ((tile_dimensions.width + x_padding) * x) as f32,
                    ((tile_dimensions.height + y_padding) * y) as f32,
                );

                sprites.push(Rect {
                    min: rect_min,
                    max: Vec2::new(
                        rect_min.x() + tile_dimensions.width as f32,
                        rect_min.y() + tile_dimensions.height as f32,
                    ),
                });
            }
        }

        SpriteSheet {
            texture,
            dimensions: Dimension2::new(columns as u32, rows as u32),
            size: Dimension2::new(
                ((tile_dimensions.width + x_padding) * columns) - x_padding,
                ((tile_dimensions.height + y_padding) * rows) - y_padding,
            ),
            sprites,
            sprite_handles: None,
        }
    }

    /// Constructs a new sprite sheet.
    pub fn new<D: Into<Dimension2>>(
        texture: Handle<Texture>,
        tile_dimensions: D,
        columns: u32,
        rows: u32,
    ) -> SpriteSheet {
        let tile_dimensions = tile_dimensions.into();
        Self::with_padding(
            texture,
            tile_dimensions,
            columns,
            rows,
            Dimension2::new(0, 0),
        )
    }

    /// Returns a builder which can be used to construct a more elaborate sprite
    /// sheet with multiple parameters to tailor it.
    ///
    /// Use this if the aim is to combine multiple sprite sheets or to combine
    /// loose sprites.
    pub fn builder() -> SpriteSheetBuilder {
        SpriteSheetBuilder::default()
    }

    /// The number of the sprites in the sprite sheet, also known as the length.
    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    /// If the sprite sheet contains sprites or not.
    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    /// Retrieves the sprite's index from a given texture and point.
    pub fn get_sprite_index<P: Into<Point2>>(
        &self,
        texture: &Handle<Texture>,
        point: P,
    ) -> SpriteSheetResult<Option<usize>> {
        let point: Point2 = point.into();
        let index = self.dimensions.encode_point(point)?;
        Ok(self.sprite_handles.as_ref().and_then(|sprite_handles| {
            sprite_handles
                .get(texture)
                .and_then(|indexes| indexes.get(&index).cloned())
        }))
    }
}
