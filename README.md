# bevy_chunk_tiles
Chunk tiles for Bevy game engine.

![](assets/img/example.png)

### Warnings
* This is still very early and experimental. The API will likely change until 0.1 release without 
notice.
* This uses a very early stage game engine.
* This is using latest bleeding edge Bevy engine while should have a 0.3 release very soon. When that
happens, this warning will be removed.

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
bevy_chunk_tiles = { git = "https://github.com/joshuajbouw/bevy_chunk_tiles", branch = "master" }
```

## Live Example
There will be more work done on examples in the very near future. For now, you can check out a quick
but non-interactive example with:
```
cargo run --example random_dungeon
```