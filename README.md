# WGPU Playground

This is a playground for [wgpu-rs](https://github.com/gfx-rs/wgpu) as I learn and experiment with graphics programming

## Crates

### [learn-wgpu](crates/learn_wgpu/README.md)

Follow along tutorial with [Learn Wgpu Tutorial](https://sotrh.github.io/learn-wgpu/). This was my starting point for getting familiar with wgpu. It is incomplete, but I highly recommend this tutorial for beginners. I constantly refer back to it for advice and help

### [game_of_life](crates/game_of_life_sim/README.md)

Implementation of [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life). This goes beyond just Conway's game. I implemented a few different rulesets from a self derived DSL (domain specific language). I also experimented with parsing wgsl code and adding a much need "#import" feature to wgsl to allow to pull in external wgsl files.
