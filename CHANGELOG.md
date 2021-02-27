# Changelog

All notable changes to this project will be documented in this file. Usually 
just additions, fixes as well as if the API had been added to.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

* Added `auto_spawn` to the `Tilemap` [#94](https://github.com/joshuajbouw/bevy_tilemap/pull/94)

## [0.3.1] - 2021-01-12

### Added

* Added bevy_log logging [#88](https://github.com/joshuajbouw/bevy_tilemap/pull/88)

### Fixed

* Fixed docs.rs fail [#89](https://github.com/joshuajbouw/bevy_tilemap/pull/89)
* Added Types feature back in [#87](https://github.com/joshuajbouw/bevy_tilemap/pull/87)

## [0.3.0] - 2021-01-11

### Added

* Auto configuration of optimal chunk sizes to texture size as well as checks
to ensure that the tiles are divisible into each other had been added. This can
be enabled in the `TilemapBuilder` with `auto_configure`.
* Auto chunk which will create new chunks automatically if you push a tile into
it. This can be enabled in the `TilemapBuilder` with `auto_chunk`
* Optional dimension 2D and 3D API.
* `TilemapDefaultPlugins` was added.
* `point` and `dimension` modules were moved to `bevy_tilemap_types` crate but 
still accessible as normal and optional.
* Example `stress_dwarves` has been added to benchmark and stress test.
* Various hex orientations were added (thanks @jamadazi!).
* A hex example `random_world` was added to showcase one of the hex 
orientations.
* `Tilemap::get_tile` method was added to get a reference to a tile.
* `Tilemap::get_tile_mut` method was added to get a mutable reference to a tile.
This should make it easier to do animations.
* `Tilemap::tile_to_chunk_point` method is now `point_to_chunk_point`.
* `tile:RawTile` is now public API but not included in the prelude as it is not
meant to be constructed.
* Examples for all tile orientations.
* Library is able to work with WASM.
* Default plugin for Tilemap was introduced.

### Changed

* The whole project was mostly refactored.
* `ChunkTilesPlugin` is now `Tilemap2DPlugin`.
* `TilemapBuilder::build()` is now `TilemapBuilder::finish()` to be consistent.
* Point module was now made optional.
* Changed the `random_dungeon` example to be more like an actual implementation.
* `Tile` had all generics removed from it.
* `Tilemap::new_chunk` is now `Tilemap::insert_chunk` to reflect the storage
internally.
* It is now required to specify if chunks are to be auto created in the 
`TilemapBuilder` with `auto_chunk` method.
* `Tilemap::remove_tile` was renamed to `clear_tile`. This makes more sense as 
it *may* be deleted if it is a sparse tile, else it is simply cleared if it is
dense.
* `Tilemap::contains_chunk` method was added to check if the tilemap contains a
chunk or not already.
* `TilemapComponents` renamed to `TilemapBundle` to stay inline with Bevy API.
* `ChunkComponents` renamed to `ChunkBundle` to stay inline with Bevy API.
* All examples were updated for latest bevy.


### Removed

* `Point2` and `Point3` deprecations were removed.

### Known Bugs

* Examples moved to its own library temporarily. This is a Bevy 0.4 issue where
you can not place `bevy` into the dev-dependencies of the Cargo.toml.
* Hex Y-axis is not perfectly centred.

## [0.2.2] - 2020-11-23

### Added

* `Tilemap::clear_tile` was added to easily clear a single tile.
* `Tilemap::clear_tiles` likewise will clear an array of tiles.
* `Point2` now implements `From<&Point2>`.
* `Point3` now implements `From<&Point3>`.
* `tilemap::Tilemap::set_tiles` now implements 
`IntoIterator<Item = ((i32, i32, i32), Tile)>` which had broken the previous 
compatibility.
* `tile::Tile` had the `non_exhaustive` derive added it now that the fields are
all public.
* `tile::Tile` added methods `with_tint` and `with_tint_and_sprite_order`.
* `chunk_update_system` was added internally to manage chunks and to get them to
update if needed.

### Changed

* `random_dungeon` example was updated to be interactive.
* `Tilemap::set_tiles` was changed to take in a `IntoIterator<Item = Tile<P, C>`,
where `C` is `Into<Color>` and P is `Into<Point2>`, from 
`IntoIterator<Item = (i32, i32, i32), Tile>`.
* `tile::Tile` was changed to include `sprite_order`, `sprite_index`, and `point`.
Field `color` is now `tint`. All fields were made public.
* `tile::Tile` methods `default`, `new` updated.
* `ChunkDimensions` was made private, it should never of been exposed but I also
very much doubt anyone would have used it.
* `map_system` was made private, it should never of been exposed.
* `add_tile` was renamed to `insert_tile`.
* `add_tiles` was renamed to `insert_tiles`.
* `clear_tile` was renamed to `remove_tile`.
* `clear_tiles` was renamed to `remove_tiles`.

**Before**
```rust
let sprite_order = 0;
let point = (1, 1, sprite_order)
let tile = Tile::new(0);
let tiles = vec![(point, tile)];
 
// defined elsewhere
tilemap.set_tiles(tiles).unwrap();
```

**After**
```rust
let point = (1, 1);
let sprite_order = 0;
let tiles = vec![Tile::new(point, sprite_order)];

// defined elsewhere
tilemap.set_tiles(tiles).unwrap();
```

* `Tilemap::set_tile` was changed to take in `Tile<P, C>`, where `P` is 
`Into<Point2>` and `C` is `Into<Color>`. This replaces the previous argument
`P` and `T` where `T` was `Into<Tile>`.

**Before**
```rust
let point = (9, 3, 0);
let sprite_index = 3;
let tile = Tile::new(sprite_index);

// defined elsewhere
tilemap.set(point, tile).unwrap();
```

**After**
```rust
let point = (9, 3);
let sprite_index = 3;
let tile = Tile::new(point, sprite_index);

tilemap.set_tile(tile).unwrap();
```

### Removed

* `tile::Tiles` was removed as it is no longer needed as all data as been set
into `Tile` to make everything easier.
* `tile::Tiles` was removed from `prelude`.
* `tile::Tile` methods `index` and `color` were removed as the fields are now
public.
* `tile::Tile` all `<Into<Tile>>` implements were removed.

## [0.2.1] - 2020-11-21

### Added

* Minimum supported rust version (MSRV) were noted to be 1.43.0 in documents.

### Fixes

* Coordinates were fixed so that 0,0 is now by default the center of the area,
not the bottom left per chunk.
* Coordinates outside of a single chunk were fixed so that they do not cause a
panic.
* Missing doc links were fixed.

### Changed

* `Tilemap::set_tiles` now implements `IntoIterator<Item = ((i32, i32, i32), Tile)>`
which had broken the previous compatibility.

**Before**
```rust
let mut tiles = Tiles::default();
for y in 0..31 {
    for x in 0..31 {
        tiles.insert((x, y, 0), 0.into());
    }
}

// Constructed Tilemap
tilemap.set_tiles(&mut tiles);
```

**After**
```rust
let mut tiles = Tiles::default();
for y in 0..31 {
    for x in 0..31 {
        tiles.insert((x, y, 0), 0.into());
    }
}

// Constructed tilemap
tilemap.set_tiles(tiles);
```

### Removed

* `Point2` impl `AsRef<(i32, i32)>` and `AsMut<(i32, i32)>` as it is not 
possible to deprecate it.

### Deprecated

* `Point2` methods `.x()` and `.y()` were deprecated to be inline with glam 
crate.
* `Point3` methods `.x()`, `.x()`, and `.z()` were deprecated to be inline with 
glam crate.

### Upgrade notes

Unfortunately some fairly big issues were released as more examples were being
made. These were fairly critical as they absolutely hindered the goals of the 
library. These have now since been fixed. Thanks to all that pointed out the
issues.

## [0.2.0] - 2020-11-20

### Added

* `ChunkDimensions` component to help the renderer know the dimensions of a 
chunk.
* `Layer` trait was added internally to implement the same methods on layers.
* `DenseLayer` and `SparseLayer` structs were added internally to provide
different methods of storing tiles.
* `LayerKind` enum was added to help specify what kind of layer needs to be
created.
* `LayerKindInner` enum was added internally to help wrap `DenseLayer` and 
`SparseLayer`.
* `Chunk` struct was added internally to store its location and sprite layers.
* `ChunkComponents` entity was added internally to help spawn chunks.
* `TilemapComponents` entity was added to help spawn tilemaps.
* `prelude` module was added versioned.
* `ChunkMesh` was added to add meshes to rendered layers.
* `point` module was added publicly with `Point2` and `Point3` structs which 
help with various operations to do with coordinates. They are not required to
be used at all.
* Render pipelines were added with GLSL shader files.
* `Tile` struct was added publicly which stores the index value of a texture
in a texture atlas.
* `Tiles` a type for `Hashmap<(i32, i32, i32), Tile>` was added publicly. Helps
to set tiles in bulk.
* `TilePoints` was added privately which is similar to `Tiles` but stores the
new `Point3`s instead.
* `dense_tiles_to_attributes` and `sparse_tiles_to_attributes` were added to 
help turn layers into attributes for the renderer.
* `Tilemap` was added with a variety of new public API to help construct new
tilemaps.
* `TilemapBuilder` factory was added to help construct new `Tilemap`s.
* Chunks with odd dimensions can now be spawned. (Thanks @blamelessgames!)

### Changed

* Updated `random_dungeon` example to use latest features.
* Changed `serde` to be and optional feature.
* Changed `Dimension2` and `Dimension3` into a struct and changed them
extensively. They were also made into private API.
* Changed `map_system` had been updated to accommodate for the new `Tilemap`.

### Broken API

* `Chunk` trait was made into a struct and vastly changed as well as made 
internal.
* `WorldChunk` struct was removed entirely.
* `WorldMap` struct was removed entirely.
* `Tile` trait was removed and replaced with a `Tile` struct.
* `coord` module was removed entirely.
* `dimension` module was made internal.
* `Dimension2` and `Dimension3` traits were made into structs and made internal.
* `ChunkTilesPlugin` had all the generic traits removed, replaced with structs.
This makes it easier for people to use the library and encourages others to
help contribute instead of keeping all the sweet updates to themselves. In the
future, generic traits will likely be brought back.
* `DimesionError` and `DimensionResult` were made private.
* `MapError` was renamed to `TilemapError` and made private. 
* `MapResult` was renamed to `TilemapResult` and made private.
* `MapEvent` was renamed to `TilemapEvent` and made private.

### Upgrade Notes

The breakpocalypse is here... But with good reasons why!

This was the actual update that made tilemap into mostly what was intended to be
on release however, due to the clear early need a naive version was pushed for
v0.1.0. Before, everything was rendered from textures that were made by the CPU
for use with the GPU. Through proper research, help from the Bevy community, and
education on GLSL shaders, shaders were created to offload all the work onto the
GPU.

The huge downside though is that pretty much 98% of the API had been broken
entirely. So, extra time and effort was put in this time to ensure that the API
will be stable from now on and going forward. Proper deprecation warnings and 
everything will be done from here on.

It was really considered to bring back all the API but, that would have been
too much work from here and onwards. Plus, it was warned that the API will be
broken between v0.1 and v0.2.

### Key points

- No more CPU work, all on the GPU with GLSL shaders.
- Huge breaking API changes everywhere, oh my! (I warned it would happen :)...)
- All traits removed, replaced with structs for early day simplicity.
- Dense and sparse layers were added to cut down on data use and increase 
performance where necessary.

### Thanks

Thank you so much @alec-deason, @alexschrod and @superdump for all your feedback, suggestions
and help understanding everything needed for this release.

## [0.1.0] - 2020-11-04

### Notes
* Rebranded from `bevy_chunk_tiles` to `bevy_tilemap`.
* `TileSetter` had a `push_stack` method added to it which allows for a whole 
`Vec<Tile>`. This is then rendered from index 0. For example, if you want to 
render a character sprite with a tile background you would push the tile in 
first then add in the character after. It is recommended to track if the floor
tile in that previous example had something on top of it before or not to cut 
down on pushing the floor tile twice, which is wasteful.

### Added
- [Tile texture stack](https://github.com/joshuajbouw/bevy_tilemap/commit/d91f9a97645d7f7692ccd532fc3cb941c0c58764)
- [Multi-thread map events](https://github.com/joshuajbouw/bevy_tilemap/commit/3312090ae3eba9a8e7edf87aaaf63d1cf96ecc07)
- [Initial commit](https://github.com/joshuajbouw/bevy_tilemap/commit/764b79e037b292d473220f43d9c8776ce626d6cb)

### Fixes
- [Fix obscure `no_implicit_prelude` issue with serde.](https://github.com/joshuajbouw/bevy_tilemap/commit/33c1317d65be3c1a0d511a2527745046cfd273fb)
- [Remove dangerous access to tiles mutably](https://github.com/joshuajbouw/bevy_tilemap/commit/90cc791a4f3d8f36421a01451020fbc927e226b2)

## [0.1.0-pre] - 2020-11-03

### Notes
* Initial release
