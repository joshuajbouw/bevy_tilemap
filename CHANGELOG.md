# Changelog

All notable changes to this project will be documented in this file. Usually 
just additions, fixes as well as if the API had been added to.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2020-11-04

### Notes
This is mostly just a document fix minor patch with nothing else too 
substantial. It would have been avoided if it could have been but alas, crates
does not allow you to republish the same version. 

### Fixes
* [Fixed external doc leak](https://github.com/joshuajbouw/bevy_tilemap/commit/b277f55bf73535a01537e2f775702226cb33178b)

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