# Voxel Generation - Rust

This is a voxel generation example in Rust using the Bevy game engine.

## Branch

This branch is a WIP rewrite. I decided to re-write this to implement entities & a server. Also it include much more optimizations like using a 1d array and 3d chunks. I will also try to make it more modular and easier to understand.

## Running

To run this example, you need to have Rust installed. You can install Rust [here](https://www.rust-lang.org/tools/install).

Once you have Rust installed, you can run the example with the following command:

```bash
cargo run --release
```

## Controls

- `WASD` - Move
- I have no idea what are the other controls, I'll add them here when i create the player controller

## Screenshots (WIP)

## TODO

- [x] Implement a crude cube spawning system thingy
- [x] Optimize it using a chunk system and meshing
- [ ] Fix chunk borders (don't render the faces that are touching other chunks but connect them seamlessly)
- [ ] Add more textures (grass, dirt, stone, etc.) (correct orientation)
- [ ] Add a skybox
- [ ] Add a surface generator
- [ ] Multithreading (chunk generation, meshing, etc.)
- [ ] Add a player controller (first person)
- [ ] Blend the surface generator nicely
- [ ] Add block breaking and placing
- [ ] Add a UI

## License

You can do whatever you want with this code. I don't care. I hope it helps you in some way.
