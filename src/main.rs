extern crate ggez;
extern crate rand;
extern crate nalgebra as na;

// extern crate ggez_goodies;

use ggez::GameResult;
use ggez::audio;
use ggez::conf;
use ggez::game;
use ggez::event::*;
use ggez::graphics;
use ggez::graphics::Color;
use ggez::graphics::Drawable;

use std::vec;
use std::time::Duration;
use std::boxed::Box;
use std::cmp::{min, max};

mod ship;
use ship::Ship;
use ship::Buttons;


// SDL2 drawing on Windows appears to be *way*
// slower than on Linux or Mac.  Ick.
#[cfg(target_os = "windows")]
const FIELD_WIDTH: usize = 40;
#[cfg(target_os = "windows")]
const FIELD_HEIGHT: usize = 30;
#[cfg(target_os = "windows")]
const FIELD_CELL_SIZE: u32 = 20;


#[cfg(not(target_os = "windows"))]
const FIELD_WIDTH: usize = 80;
#[cfg(not(target_os = "windows"))]
const FIELD_HEIGHT: usize = 60;
#[cfg(not(target_os = "windows"))]
const FIELD_CELL_SIZE: u32 = 10;

fn screen_to_field_coords(x: u32, y: u32) -> (usize, usize) {
    let xn = (x / FIELD_CELL_SIZE) as usize;
    let yn = (y / FIELD_CELL_SIZE) as usize;
    (xn, yn)
}

fn field_to_screen_coords(x: usize, y: usize) -> (i32, i32) {
    let xn = (x as u32) * FIELD_CELL_SIZE;
    let yn = (y as u32) * FIELD_CELL_SIZE;
    (xn as i32, yn as i32)
}


fn interp_between_square(t: f64, v1: Color, v2: Color) -> Color {

    let (r1, g1, b1, a1) = v1.rgba();
    let (fr1, fg1, fb1, fa1) = (r1 as f64, g1 as f64, b1 as f64, a1 as f64);

    let (r2, g2, b2, a2) = v2.rgba();
    let (fr2, fg2, fb2, fa2) = (r2 as f64, g2 as f64, b2 as f64, a2 as f64);

    let dr = fr2 - fr1;
    let dg = fg2 - fg1;
    let db = fb2 - fb1;
    let da = fa2 - fa1;

    let t2 = f64::sqrt(t);
    let (rr, rg, rb, ra) = (fr1 + dr * t2, fg1 + dg * t2, fb1 + db * t2, fa1 + da * t2);
    Color::RGBA(rr as u8, rg as u8, rb as u8, ra as u8)
}

fn clamp(val: f32, lower: f32, upper: f32) -> f32 {
    f32::min(f32::max(val, lower), upper)
}

// Fields values are 0 to +1
// Color values are 0-255
// We'll do negative = red and positive = blue
fn field_to_color(val: f32) -> Color {
    let black = Color::RGBA(0, 120, 255, 255);
    let negative_max = Color::RGBA(0, 70, 128, 255);
    let positive_max = Color::RGBA(150, 200, 255, 255);
    if val < 0.0 {
        interp_between_square(-val as f64, black, negative_max)
    } else {
        interp_between_square(val as f64, black, positive_max)
    }
}

struct WaveImages {
    image: graphics::Image,
    layers: Vec<graphics::Rect>,
}

const FLIP_THRESHOLD: f32 = 0.1;

impl WaveImages {
    fn new(ctx: &mut ggez::Context) -> Self {
        let img = graphics::Image::new(ctx, "ocean_tiles.png").unwrap();
        let layers = vec![graphics::Rect::new(128, 0, 128, 128),
                          graphics::Rect::new(0, 0, 128, 128),
                          graphics::Rect::new(128, 128, 128, 128),
                          graphics::Rect::new(0, 128, 128, 128)];
        WaveImages {
            image: img,
            layers: layers,
        }
    }

    fn draw_images(&mut self, ctx: &mut ggez::Context, rect: graphics::Rect, height: f32) {
        // let c = field_to_color(height);
        // self.image.set_color_mod(c);
        let img = if height < -FLIP_THRESHOLD {
            self.layers[0]
        } else if height <= 0.0 {
            self.layers[1]
        } else if height <= FLIP_THRESHOLD {
            self.layers[2]
        } else {
            self.layers[3]
        };

        let _ = self.image.draw(ctx, Some(img), Some(rect));
    }
}

#[derive(Copy, Clone, Debug)]
struct WaveType {
    velocity: f32,
    position: f32,
}

impl WaveType {
    fn new(position: f32) -> Self {
        WaveType {
            velocity: 0.0,
            position: position,
        }
    }

    fn restoring_force(&self) -> f32 {
        -self.position * 0.05
        // 0.0
    }
}

impl Default for WaveType {
    fn default() -> Self {
        // If you make these 0 the optimizer/OS won't
        // actually allocate space for the arrays it needs
        // until the game is running, I suspect.
        // So it gets laggy for the first few seconds.
        // With a slight offset it APPEARS to MOSTLY fix the problem.
        WaveType {
            velocity: 0.001,
            position: 0.001,
        }
    }
}

struct Field(Vec<Vec<WaveType>>);

impl Field {
    fn new() -> Self {
        let mut field = Vec::with_capacity(FIELD_WIDTH);
        for _i in 0..FIELD_WIDTH {
            let mut bit = Vec::with_capacity(FIELD_HEIGHT);
            bit.resize(FIELD_HEIGHT, WaveType::default());
            field.push(bit);
        }
        Field(field)
    }

    fn draw(&mut self, ctx: &mut ggez::Context, waves: &mut WaveImages) -> GameResult<()> {
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                let (xi, yi) = field_to_screen_coords(x, y);
                let r = graphics::Rect::new(xi, yi, FIELD_CELL_SIZE, FIELD_CELL_SIZE);
                let color = field_to_color(self.0[x][y].position);
                graphics::set_color(ctx, color);
                // Wow actually putting a ? at the end of this takes us
                // from 325 to 275 fps.  Wacky.
                let _ = graphics::rectangle(ctx, graphics::DrawMode::Fill, r);

                // let color = waves.draw_images(ctx, r, self.0[x][y].position);
            }
        }

        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                let (xi, yi) = field_to_screen_coords(x, y);
                let r = graphics::Rect::new(xi, yi, FIELD_CELL_SIZE, FIELD_CELL_SIZE);
                // let color = field_to_color(self.0[x][y].position);
                // graphics::set_color(ctx, color);
                // graphics::rectangle(ctx, graphics::DrawMode::Fill, r);

                waves.draw_images(ctx, r, self.0[x][y].position);
            }
        }
        Ok(())
    }

    fn update(&mut self) {
        // self.sprinkle_random_bits();
        self.propegate();
        self.decay();
    }

    fn decay(&mut self) {
        // Decay intensity.
        // Setting this to 0.98 makes the wave go forever,
        // setting it to 0.97 makes it just kind of go plonk.
        // At least with a surface tension of 3.0.
        let decay_factor = 0.99;
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                // let val = self.0[x][y].position * decay_factor;
                // self.0[x][y].position = val;
                // Decaying position vs. velocity doesn't seem
                // to have made much difference
                self.0[x][y].velocity *= decay_factor;
                self.0[x][y].position *= decay_factor;

                // We might just want to zero this out if it goes below a certain point.
                // if f32::abs(self.0[x][y].velocity) < 0.001 {
                //     self.0[x][y].velocity = 0.0;
                // }
                // if f32::abs(self.0[x][y].position) < 0.001 {
                //     self.0[x][y].position = 0.0;
                // }
            }
        }
    }

    // This gets the difference between a poitn and one of its neighbors.
    //
    fn relative_position(&self, x: i32, y: i32, dx: i32, dy: i32) -> f32 {
        let pos = self.0[x as usize][y as usize].position;
        if x == 0 && dx < 0 {
            0.0
        } else if x == (FIELD_WIDTH as i32) - 1 && dx > 0 {
            0.0
        } else if y == 0 && dy < 0 {
            0.0
        } else if y == (FIELD_HEIGHT as i32) - 1 && dy > 0 {
            0.0
        } else {
            self.0[(x + dx) as usize][(y + dy) as usize].position - pos

        }
    }

    fn propegate(&mut self) {
        let dt = 0.01;
        let sqrt2 = std::f32::consts::SQRT_2;
        // How strongly each cell is affected by its neighbors.
        // Higher numbers mean weaker.
        let surface_tension = 4.0;
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                let mut val = self.0[x][y];
                let ix = x as i32;
                let iy = y as i32;

                val.position += val.velocity * dt;
                // val.position = clamp(val.position, -1.0, 1.0);
                // total force = restoring force plus a force based on the
                // sum of differences in position  between itself and its
                // neighbors
                // We can add divisors or multipliers based on the position
                // to mess with the "speed of sound", kinda, or at least make
                // anisotropic substances.  Sweet!
                let neighbor_force = self.relative_position(ix, iy, 0, -1) +
                                     self.relative_position(ix, iy, 0, 1) +
                                     self.relative_position(ix, iy, -1, 0) +
                                     self.relative_position(ix, iy, 1, 0) +
                                     self.relative_position(ix, iy, -1, -1) / sqrt2 +
                                     self.relative_position(ix, iy, 1, -1) / sqrt2 +
                                     self.relative_position(ix, iy, -1, 1) / sqrt2 +
                                     self.relative_position(ix, iy, 1, 1) / sqrt2;
                let forces = val.restoring_force() + neighbor_force / surface_tension;
                val.velocity += forces;
                val.velocity = clamp(val.velocity, -1.0, 1.0);

                // println!("{:?}", val);
                self.0[x][y] = val;
            }
        }
    }

    // Creates a square disturbance in the field, setting all positions
    // inside it to the given force.
    // Eventually should add the values, not set them.
    // Maybe should set velocity rather than position?
    fn create_splash(&mut self, x: usize, y: usize, radius: usize, force: f32) {
        let max_x = min(x + radius, FIELD_WIDTH);
        let min_x = max(x - radius, 0);
        let max_y = min(y + radius, FIELD_HEIGHT);
        let min_y = max(y - radius, 0);
        // println!("{}:{}, {}:{}", min_x, max_x, min_y, max_y);
        for x in min_x..max_x {
            for y in min_y..max_y {
                // println!("Setting cell {},{} to force {}", x, y, force);
                // Setting position vs. velocity doesn't appear to make
                // much difference.
                // ...okay, the position makes bigger waves.
                // self.0[x][y].position = force;
                self.0[x][y].position += force;
            }
        }
    }

    pub fn read_strength(&self, x: i32, y: i32) -> f32 {
        let x = x as u32;
        let y = y as u32;
        self.0[x as usize][y as usize].position
        // f32::abs(self.0[x as usize][y as usize].position)
    }

    pub fn read_strength_area(&self, x: i32, y: i32) -> (f32, f32) {
        let radius = 2;
        let x = x as u32;
        let y = y as u32;
        let mut max = 0.0;
        let mut min = 0.0;
        for xi in (x - radius)..(x + radius) {
            for yi in (y - radius)..(y + radius) {
                let value = self.0[x as usize][y as usize].position;
                max = f32::max(value, max);
                min = f32::min(value, min);
            }
        }
        (max, min)
        // f32::abs(self.0[x as usize][y as usize].position)
    }

    #[allow(dead_code)]
    fn sprinkle_random_bits(&mut self) {
        let tx = rand::random::<usize>() % FIELD_WIDTH;
        let ty = rand::random::<usize>() % FIELD_HEIGHT;
        self.0[tx][ty].position = 1.0;
    }
}


// The ndarray crate would be nice here.
struct MainState {
    field: Field,
    player1: Ship,
    player2: Ship,
    frame: usize,
    wave_images: WaveImages,
    player1_wins_image: graphics::Image,
    player2_wins_image: graphics::Image,
}

impl MainState {
    fn new(ctx: &mut ggez::Context) -> Self {
        let f = Field::new();
        let wi = WaveImages::new(ctx);
        let player1_wins_image = graphics::Image::new(ctx, "ship1_wins.png").unwrap();
        let player2_wins_image = graphics::Image::new(ctx, "ship2_wins.png").unwrap();
        MainState {
            field: f,
            player1: Ship::new(100 as i32, 100 as i32, ctx, "ship1"),
            player2: Ship::new(600 as i32, 400 as i32, ctx, "ship2"),
            frame: 0,
            wave_images: wi,
            player1_wins_image: player1_wins_image,
            player2_wins_image: player2_wins_image,
        }
    }

    fn calculate_flips(&mut self) {
        let ship_location1 = self.player1.location;
        let wave_location1 = screen_to_field_coords(ship_location1.x as u32,
                                                    ship_location1.y as u32);
        let (wave_strength1, _) = self.field
            .read_strength_area(wave_location1.0 as i32, wave_location1.1 as i32);
        if wave_strength1 > FLIP_THRESHOLD && !self.player1.jumping {
            self.player1.flip();
        }

        let ship_location2 = self.player2.location;
        let wave_location2 = screen_to_field_coords(ship_location2.x as u32,
                                                    ship_location2.y as u32);
        // println!("Location 1: {:?}, location 2: {:?}",
        //          wave_location1,
        //          wave_location2);
        let (_, wave_strength2) = self.field
            .read_strength_area(wave_location2.0 as i32, wave_location2.1 as i32);
        if wave_strength2 < -FLIP_THRESHOLD && !self.player2.jumping {
            self.player2.flip();
        }

        println!("Flipped? {} {}", self.player1.flipped, self.player2.flipped);

        println!("wave_strength1 = {:+} wave_strength2 = {:+}",
                 wave_strength1,
                 wave_strength2);
    }
}


impl game::EventHandler for MainState {
    fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<()> {

        // Add a wake as the ship moves
        let p1_field_location = screen_to_field_coords(self.player1.location.x as u32,
                                                       self.player1.location.y as u32);


        let p2_field_location = screen_to_field_coords(self.player2.location.x as u32,
                                                       self.player2.location.y as u32);
        let (sx1, sy1) = p1_field_location;
        let (sx2, sy2) = p2_field_location;

        self.field.update();
        self.player1.update();
        self.player2.update();

        if self.player1.post_jump == 30 {
            // create splash from landing
            // println!("Splashing down");
            self.field.create_splash(sx1, sy1, 6, -1.0);
        } else if !self.player1.jumping {
            // create wake
            self.field.create_splash(sx1, sy1, 1, -0.01);
        }

        if self.player2.post_jump == 30 {
            self.field.create_splash(sx2, sy2, 6, 1.0);
        } else if !self.player2.jumping {
            self.field.create_splash(sx2, sy2, 1, 0.01);
        }

        if self.frame % 100 == 0 {
            let time = ggez::timer::get_time_since_start(ctx).as_secs();
            println!("Time {}s Frame {}, FPS: {}",
                     time,
                     self.frame,
                     ggez::timer::get_fps(ctx));
        }

        // Shipwave
        // println!("Wave at ship {}", self.field.read_strength(self.ship.location.x as i32,
        //    self.ship.location.y as i32));

        self.calculate_flips();

        self.frame += 1;
        // println!("Frame {}, FPS: {}", self.frame, ggez::timer::get_fps(ctx));

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        graphics::clear(ctx);

        // Background
        self.field.draw(ctx, &mut self.wave_images)?;

        // Foreground
        self.player1.draw(ctx)?;
        self.player2.draw(ctx)?;

        if self.player1.flipped {
            self.player2_wins_image.draw(ctx, None, None)?;
        } else if self.player2.flipped {
            self.player1_wins_image.draw(ctx, None, None)?;

        }

        ctx.renderer.present();
        Ok(())
    }

    fn key_down_event(&mut self, _keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match _keycode {
            Keycode::W => self.player1.key_down_event(Buttons::Up),
            Keycode::A => self.player1.key_down_event(Buttons::Left),
            Keycode::D => self.player1.key_down_event(Buttons::Right),
            Keycode::S => self.player1.jump(),

            Keycode::I => self.player2.key_down_event(Buttons::Up),
            Keycode::J => self.player2.key_down_event(Buttons::Left),
            Keycode::L => self.player2.key_down_event(Buttons::Right),
            Keycode::K => self.player2.jump(),
            _ => (),
        }

    }


    fn key_up_event(&mut self, _keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match _keycode {
            Keycode::W => self.player1.key_up_event(Buttons::Up),
            Keycode::A => self.player1.key_up_event(Buttons::Left),
            Keycode::D => self.player1.key_up_event(Buttons::Right),

            Keycode::I => self.player2.key_up_event(Buttons::Up),
            Keycode::J => self.player2.key_up_event(Buttons::Left),
            Keycode::L => self.player2.key_up_event(Buttons::Right),
            _ => (),
        }
    }

    fn controller_button_down_event(&mut self, _btn: Button) {
        println!("Button {:?} released", _btn);
        //     let x = x as u32 / FIELD_CELL_SIZE;
        //     let y = y as u32 / FIELD_CELL_SIZE;
        //     println!("Creating splash at {}, {}", x, y);
        // match button {
        //     MouseButton::Left => {
        //         self.player2.key_down_event(Buttons::Up);
        //     }
        //     // MouseButton::Left => {
        //     //     self.field.create_splash(x as usize, y as usize, 3, 1.0);
        //     // }
        //     //
        //     // MouseButton::Right => {
        //     //     self.field.create_splash(x as usize, y as usize, 3, -1.0);
        //     // }
        //     _ => (),
        // }
    }
    fn mouse_button_down_event(&mut self, _button: MouseButton, _x: i32, _y: i32) {
        // println!("Mouse clicking at {}, {}", x, y);
        // let x = x as u32 / FIELD_CELL_SIZE;
        // let y = y as u32 / FIELD_CELL_SIZE;
        // println!("Creating splash at {}, {}", x, y);
        // match button {
        //     MouseButton::Left => {
        //         self.field.create_splash(x as usize, y as usize, 3, 1.0);
        //     }
        //     _ => (),
        // }
    }

    fn controller_button_up_event(&mut self, _btn: Button) {
        println!("Button {:?} pressed", _btn);
        //     let x = x as u32 / FIELD_CELL_SIZE;
        //     let y = y as u32 / FIELD_CELL_SIZE;
        //     println!("Creating splash at {}, {}", x, y);
        // match button {
        //     MouseButton::Left => {
        //         self.player2.key_up_event(Buttons::Up);
        //     }
        //     // MouseButton::Left => {
        //     //     self.field.create_splash(x as usize, y as usize, 3, 1.0);
        //     // }
        //     //
        //     // MouseButton::Right => {
        //     //     self.field.create_splash(x as usize, y as usize, 3, -1.0);
        //     // }
        //     _ => (),
        // }
    }

    fn controller_axis_event(&mut self, axis: Axis, value: i16) {
        println!("Axis {:?}, value {}", axis, value);
        // if xrel < 0 {
        //     self.player2.key_up_event(Buttons::Right);
        //     self.player2.key_down_event(Buttons::Left);
        // } else {
        //     self.player2.key_up_event(Buttons::Left);
        //     self.player2.key_down_event(Buttons::Right);

        // }
    }
}

fn default_conf() -> conf::Conf {
    let mut c = conf::Conf::new();
    c.window_title = String::from("Flipwrecked");
    // c.window_width
    c
}

struct TitleScreen {
    image: graphics::Image,
    done: bool,
}

impl TitleScreen {
    fn new(ctx: &mut ggez::Context) -> Self {
        TitleScreen {
            image: graphics::Image::new(ctx, "title.png").unwrap(),
            done: false,
        }
    }
}

impl game::EventHandler for TitleScreen {
    fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<()> {
        if self.done {
            ctx.quit()?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        graphics::clear(ctx);

        self.image.draw(ctx, None, None)?;

        ctx.renderer.present();
        Ok(())
    }


    fn key_down_event(&mut self, _keycode: Keycode, _keymod: Mod, _repeat: bool) {
        // End this gameloop and move on to the next.
        self.done = true;
    }
}

fn main() {
    let c = default_conf();
    let mut ctx = ggez::Context::load_from_conf("Flipwrecked", c).unwrap();

    let titlescreen = TitleScreen::new(&mut ctx);
    let g = game::Game::from_state(ctx, titlescreen);
    let mut ctx = g.run().unwrap();

    let m = audio::Music::new(&mut ctx, "Trance.ogg").unwrap();
    audio::play_music(&mut ctx, &m).unwrap();
    let state = MainState::new(&mut ctx);
    let g = game::Game::from_state(ctx, state);

    g.run().unwrap();
}
