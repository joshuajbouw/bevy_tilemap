# bevy_tilemap
[![Crates.io](https://img.shields.io/crates/v/bevy_tilemap.svg)](https://crates.io/crates/bevy_tilemap)
[![Crates.io](https://img.shields.io/crates/d/bevy_tilemap.svg)](https://crates.io/crates/bevy_tilemap)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/joshuajbouw/bevy_tilemap/blob/master/LICENSE)
[![Rust](https://github.com/joshuajbouw/bevy_tilemap/workflows/CI/badge.svg)](https://github.com/joshuajbouw/bevy_tilemap/actions)

Chunk based tilemap for Bevy game engine.

![](assets/img/example.png)

### Warnings
* This is still very early and experimental and uses a very new game engine.

## Features
* Multi-threaded chunk based tile maps
* Generic traits to be easy to implement into any system
* Helpful traits that help encode and decode coordinates into indexe

## Design 
This is not intended to be just another Tile Map. It is meant to be a framework and extensible by
design, like Bevy. There is an emphasis placed on generic traits to accomplish that. As well as
work done to keep it as close to Bevy API as possible. This allows anyone to create their own tiles, 
chunks and maps and still retain the speed of a handcrafted multi-threaded chunk loader and tile map.

## Usage
Once Bevy 0.3 is released, this will be published.

Add to your `Cargo.toml` file:
```toml
[patch.crates-io]
bevy = { git = "https://github.com/bevyengine/bevy", branch = "master" }

[dependencies]
bevy = 0.2
bevy_tilemap = { git = "https://github.com/joshuajbouw/bevy_tilemap", branch = "master" }
```

## Live Example
There will be more work done on examples in the very near future. For now, you can check out a quick
but non-interactive example with:
```
cargo run --example random_dungeon
```