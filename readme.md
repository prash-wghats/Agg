# Agg Graphic Library [Rust]
Rust port of [Anti-Grain Geometry library] (https://agg.sourceforge.net/antigrain.com/) (version 2.4)
written by Maxim Shemanarev in C++.

## To build:
```
cargo build
```
## To build/run examples:
```
cargo run --example blend_color --features "sdl"
cargo run --example aa_demo --features "win32"
cargo run --example blend_color --features "x11"
cargo run --example gpc_test --features "win32","libgpc"
cargo build --examples --features "win32","libgpc"
```
 The image/data files required by examples are in ```examples/web/* ```