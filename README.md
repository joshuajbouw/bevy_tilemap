<h1 align="center">Bevy Tilemap</h1>

<p align="center">
    <img width="600"
        alt="Bevy Tilemap logo"
        src="https://github.com/joshuajbouw/bevy_tilemap/raw/master/docs/img/logo.gif">
</p>

<p align="center">
    <a href="https://github.com/joshuajbouw/bevy_tilemap" alt="github">
        <img src="https://img.shields.io/badge/github-joshuajbouw/bevy__tilemap-8da0cb?style=for-the-badge&logo=github" height="20"/></a>
    <a href="https://crates.io/crates/bevy_tilemap" alt="crates.io">
        <img src="https://img.shields.io/crates/v/bevy_tilemap.svg?style=for-the-badge" height="20"/></a>
    <a href="https://docs.rs/bevy_tilemap" alt="docs.rs">
        <img src="https://img.shields.io/badge/docs.rs-bevy__tilemap-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20"/></a>
    <a href="https://github.com/joshuajbouw/bevy_tilemap/actions" alt="build status">
        <img src="https://img.shields.io/github/workflow/status/joshuajbouw/bevy_tilemap/CI/master?style=for-the-badge" height="20"/></a>
    <a href="https://github.com/joshuajbouw/bevy_tilemap/blob/master/LICENSE" alt="license">
        <img src="https://img.shields.io/crates/l/bevy_tilemap.svg?style=for-the-badge" height="20"/></a>
</p>

<h4 align="center">Chunk based tilemap for Bevy game engine.</h4>

Bevy Tilemap allows for Bevy native batch-rendered tiles in maps to be 
constructed with chunk based loading, efficiently.

Simple yet refined in its implementation, it is meant to attach to other 
extensible plugins that can enhance its functionality further. Hand-crafted
tilemaps with an attentive focus on performance, and low data usage.

## WARNING
This project is still experimental and the API will likely break often before 
the first release. It uses an experimental game engine which too may break the
API. Semantic versioning will be followed as much as possible and the 
contributors will as much as they possibly can try to keep the API stable.

If you have API suggestions, now is the time to do it.

## Features
* Perfect for game jams.
* Easy to use and *mostly* stable API with thorough documentation.
* Endless or constrained dimension tilemaps.
* Batched rendering of many tiles.
* Square and hex tiles.

## Build Features
* Serde support
* Extra types

## Design 
This is not intended to be just another Tilemap. It is meant to be a framework 
and extensible by design, like Bevy. As well as work done to keep it as close to 
Bevy API as possible while keeping in mind of Rust API best practices. It is not
meant to be complicated and created to be simple to use but give enough 
functionality to advanced users.

Less time fiddling, more time building

## Usage
Add to your `Cargo.toml` file:
```toml
[dependencies]
bevy = "0.4"
bevy_tilemap = "0.4"
```

## Simple tilemap construction

At the most basic implementation, there is not a whole lot that is required to
get the tilemap going as shown below.

```rust
use bevy_tilemap::prelude::*;
use bevy::asset::HandleId;
use bevy::prelude::*;

// This must be set in Asset<TextureAtlas>.
let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());

let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);

// Coordinate point with Z order.
let point = (16, 16, 0);
let tile_index = 0;
tilemap.set_tile(point, tile_index);

tilemap.spawn_chunk_containing_point(point);
```

Of course, using the `Tilemap::builder()` this can be constructed with many more
advanced features.

* 3D and 2D tilemaps.
* Texture atlas.
* Dimensions of the tilemap.
* Dimensions of a chunk.
* Dimensions of a tile.
* Adding Z render layers
* Automated chunk creation.
* Auto-spawning of tiles based on view.

With many more features planned for future updates to bring it up to par with
other tilemap implementations for other projects.

## Future plans

There is still a lot to do but the API is now stable and should be fine for a
while now. The next release is focused on added automated methods and system.

- **Auto-tile**: Picks the right tile based around the neighbours of the tile.
- **Tile import**: Imports tiles from a file from multiple formats.

## Building

`bevy_tilemap` is only guaranteed to work from stable Rust toolchain and up. This
is to be inline with the rest of Bevy engine.

Once you have a development environment, Bevy Tilemap can be fetched using git:

```bash
$ git clone --recursive https://github.com/joshuajbouw/bevy_tilemap/
```

and then built using cargo:

```bash
$ cargo build --example random_dungeon
```

cargo can also be used to run tests:

```
$ cargo test
```
