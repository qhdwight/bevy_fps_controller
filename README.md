[![Rust](https://github.com/qhdwight/bevy_fps_controller/actions/workflows/rust.yml/badge.svg)](https://github.com/qhdwight/bevy_fps_controller/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/bevy_fps_controller)](https://crates.io/crates/bevy_fps_controller)

# Bevy FPS Controller

Inspired from Source engine movement, this plugin implements movement suitable for FPS games.

Feel free to make issues/PRs!

### Features

* Air strafing
* Bunny hopping if the jump key is held down
* Moving along sloped ground
* Crouching and sprinting
* Crouching prevents falling off ledges (Rapier only)
* Instantly clear small steps (Rapier only)
* Noclip mode
* Configurable settings

### Examples

See [minimal_rapier.rs](./examples/minimal_rapier.rs) or [minimal_avian.rs](./examples/minimal_avian.rs)

Make sure to enable either the `rapier` or `avian` feature in `Cargo.toml` depending on what your backing physics engine is.

```bash
cargo run --release --features rapier --example minimal_rapier
```

### Demo

https://user-images.githubusercontent.com/20666629/221995601-2ec352fe-a8b0-4f8c-9a81-beaf898b2b41.mp4

Used by my other project: https://github.com/qhdwight/voxel-game-rs
