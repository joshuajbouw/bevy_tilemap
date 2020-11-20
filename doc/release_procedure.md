# Release Checklist
- **API**
    - [ ] Rust lang's [API guidelines]
- **Versions**
    - [ ] Bump version [Cargo.toml]
    - [ ] Bump version doc(html_root_url) in [lib.rs]
- **Docs**
    - [ ] Double check changed function documentation.
    - [ ] Update [README.md] if needed.
    - [ ] Update [CHANGELOG.md] only significant changes. Follow this 
    [example by diesel-rs] and use the [compare function on Github] to help.
- **Publish**
    - [ ] [Publish on Github] following the template below.
    - [ ] Ensure that the release body and [CHANGELOG.md] are the same.
    - [ ] Ensure that everything is pushed.
    - [ ] Dry-run publish to crates.io with `cargo publish --dry-run` 
    - [ ] Publish to crates.io with `cargo publish`
    
[Cargo.toml]: ../Cargo.toml
[lib.rs]: ../src/lib.rs
[README.md]: ../README.md
[CHANGELOG.md]: ../CHANGELOG.md
[API guidelines]: https://rust-lang.github.io/api-guidelines/checklist.html
[example by diesel-rs]: https://github.com/diesel-rs/diesel/blob/master/CHANGELOG.md
[Publish on Github]: https://github.com/joshuajbouw/bevy_tilemap/releases/new
[compare function on Github]: https://github.com/joshuajbouw/bevy_tilemap/compare

# Release Template

Set `tile` to `Bevy Tilemap x.y.z`. If it is a pre-release, `Bevy Tilemap x.y.z pre-release x`
Set `version tag` to `vx.y.z`

```
## [x.y.z] - yyyy-mm-dd

### Added

* Bullet form listed added `Features` and `API`.

### Changed

* The way logic has changed.
* Minimum rustc versions.
* Changed traits.
* Generally everything that hadn't been added to, but were changed.

### Broken API

* Any API that had been changed. i.e arg added or removed.

### Deprecated

* Anything that had been deprecated. If it needs to be removed, deprecate it.

### Removed

* Removed deprecated items
* Support for < versioned crates. i.e: Support for `uuid` version < 0.7.0 has 
been removed.
* Any non-breaking API changes

### Fixed

* Any possible bugs or issues.

### Security

* Any vulnerabilities that had been detected and fixed.

### Upgrade Notes

Key points:

- Various listed key points that are highlights of this release.

### Thanks

* To various contributors.
```