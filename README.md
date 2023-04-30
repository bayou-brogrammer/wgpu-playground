# WGPU Playground

This is a playground for [wgpu-rs](https://github.com/gfx-rs/wgpu) as I learn and experiment with graphics programming

## Notes

This repository uses [Mold Linker](https://github.com/rui314/mold) for faster compile times with Rust™️. I highly recommend it for any Rust™️ project. If you wish to use something different, either delete the .cargo directory or change your linker within the [.cargo/config.toml](.cargo/config.toml) file. It also has commented out bindings for nightly-rust.

## Crates

### [learn-wgpu](crates/learn_wgpu/README.md)

Follow along tutorial with [Learn Wgpu Tutorial](https://sotrh.github.io/learn-wgpu/). This was my starting point for getting familiar with wgpu. It is incomplete, but I highly recommend this tutorial for beginners. I constantly refer back to it for advice and help

### [game_of_life](crates/game_of_life_sim/README.md)

Implementation of [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life). This goes beyond just Conway's game. I implemented a few different rulesets from a self derived DSL (domain specific language). I also experimented with parsing wgsl code and adding a much need "#import" feature to wgsl to allow to pull in external wgsl files.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
