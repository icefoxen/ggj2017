A Global Game Jam 2017 project by:

* Simon Heath
* Jake Stambaugh
* Carsten Huang
* Music by Daniel Cohen


# Building:

* Install Rust with rustup (stable version)
* On Debian Linux: `apt install libsdl2-dev libsdl2-mixer-dev libsdl2-image-dev`
* On Mac:
* On Windows: Should just have to copy the SDL dll's from the binary release of SDL, SDL_image and
SDL_mixer to the root directory of the project folder.  Follow instructions from
https://github.com/AngryLawyer/rust-sdl2
* Build and run with: `cargo run --release` (debug mode is super duper slow)
