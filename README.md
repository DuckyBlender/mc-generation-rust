# Voxel Generation - Rust

This is a voxel generation example in Rust using the Bevy game engine.

## Running

To run this example, you need to have Rust installed. You can install Rust [here](https://www.rust-lang.org/tools/install).

Once you have Rust installed, you can run the example with the following command:

```bash
cargo run --release
```

## Controls

- `WASD` - Move
- `Control` - Move slightly faster
- `Space` - Move up
- `Shift` - Move down

## Screenshots (WIP)

## TODO

- [x] Implement a crude cube spawning system thingy
- [x] Optimize it using a chunk system and meshing
- [x] Fix chunk borders (don't render the faces that are touching other chunks but connect them seamlessly)
- [ ] Add a surface generator and blend it using a function
- [x] Add a skybox
- [ ] Add players
- [ ] Add block breaking and placing
- [ ] Add a UI

## License

You can do whatever you want with this code. I don't care. Just don't sue me if it breaks something.
