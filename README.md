# Voxel Generation - Rust

This is a voxel generation example in Rust using the Bevy game engine.

## Running

To run this example, you need to have Rust installed. You can install Rust [here](https://www.rust-lang.org/tools/install).

Once you have Rust installed, you can run the example with the following command:

```bash
cargo run --release
```
# WARNING! RIGHT NOW THE GAME RANDOMLY CRASHES. NO IDEA WHY. I'LL FIX IT SOON. (I HOPE) (I'M NOT SURE IF IT'S A BEVY BUG OR MY CODE) (I THINK IT'S MY CODE) 
## Controls

- `WASD` - Move
- I have no idea what are the other controls, I'll add them here when i create the player controller

## Screenshots (WIP)

## TODO

- [x] Implement a crude cube spawning system thingy
- [x] Optimize it using a chunk system and meshing
- [x] Fix chunk borders (don't render the faces that are touching other chunks but connect them seamlessly)
- [x] Add more textures (grass, dirt, stone, etc.) (correct orientation)
- [x] Add a skybox
- [x] Add a surface generator
- [x] Multithreading (chunk generation, meshing, etc.)
- [ ] Add a player controller (first person) [TOP PRIORITY]
- [ ] Blend the surface generator nicely
- [ ] Add block breaking and placing
- [ ] Add a UI

## License

You can do whatever you want with this code. I don't care. Just don't sue me if it breaks something.
