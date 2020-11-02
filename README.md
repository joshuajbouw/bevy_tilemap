# bevy_chunk_tiles
Chunk tiles for Bevy game engine.

![](assets/img/example.png)

### Warning
This is still very early and experimental. The API will likely change between version 0.1 -> 0.2
without notice.

### Warning: Early Bevy Release
This is using latest bleeding edge Bevy engine while should have a 0.3 release very soon. When that
happens, this warning will be removed.

## Usage
Add to your `Cargo.toml` file:
```toml
[patch.crates-io]
bevy = { git = "https://github.com/bevyengine/bevy", branch = "master" }

[dependencies]
bevy = 0.2
bevy_chunk_tiles = 0.1
```