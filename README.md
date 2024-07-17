# Last Call BBS Dungeons & Diagrams solver

Automated solution finder and executioner for Last Call BBS Dungeons & Diagrams.

Get the game on [Steam](https://store.steampowered.com/app/1511780/Last_Call_BBS/).

So far, this gets some easy puzzles right. There's 2 issues it has with harder puzzles:
- It's really slow to collapse random tiles, but necessary if we get stuck after collapsing all structures.
- It could potentially generate wrong results because it doesn't check for all cases of all treasure room rules. 

## Requirements

I only ran this on linux so that's what I'm gonna list:

`apt-get install libxcb1 libxrandr2 libdbus-1-3 libxdo-dev`

- `libxcb1`, `libxrandr2`, `libdbus-1-3` are for the `screenshots` crate

- `libxdo-dev` is for the `enigo`


## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
