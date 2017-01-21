extern crate ggez;
extern crate rand;

// extern crate ggez_goodies;

use ggez::GameResult;
use ggez::conf;
use ggez::game;
use ggez::event::*;
use ggez::graphics;
use ggez::graphics::Color;
use ggez::graphics::Drawable;

use std::time::Duration;
use std::boxed::Box;
use std::cmp::{min, max};

mod ship;
use ship::Ship;

const FIELD_WIDTH: usize = 200;
const FIELD_HEIGHT: usize = 150;

const FIELD_CELL_SIZE: u32 = 4;

// stolen from ggez-goodies particles; we really should have a general
// interpolation functionality there.  Does nalgebra or such have one?
fn interp_between(t: f64, v1: Color, v2: Color) -> Color {

    let (r1, g1, b1, a1) = v1.rgba();
    let (fr1, fg1, fb1, fa1) = (r1 as f64, g1 as f64, b1 as f64, a1 as f64);

    let (r2, g2, b2, a2) = v2.rgba();

    let dr = (r2 - r1) as f64;
    let dg = (g2 - g1) as f64;
    let db = (b2 - b1) as f64;
    let da = (a2 - a1) as f64;

    let (rr, rg, rb, ra) = (fr1 + dr * t, fg1 + dg * t, fb1 + db * t, fa1 + da * t);
    Color::RGBA(rr as u8, rg as u8, rb as u8, ra as u8)
}

// Fields values are 0 to +1
// Color values are 0-255
// We'll do negative = red and positive = blue
fn field_to_color(val: f32) -> Color {
    let black = Color::RGBA(0, 0, 0, 255);
    let negative_max = Color::RGBA(128, 0, 0, 255);
    let positive_max = Color::RGBA(0, 0, 128, 255);
    if val < 0.0 {
        interp_between(-val as f64, black, negative_max)
    } else {
        interp_between(val as f64, black, positive_max)
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
        WaveType {
            velocity: 0.0,
            position: 0.0,
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
        let mut f = Field(field);
        f.create_splash(100, 75, 5, 1.0);
        f
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                let xi = x as i32 * FIELD_CELL_SIZE as i32;
                let yi = y as i32 * FIELD_CELL_SIZE as i32;
                let r = graphics::Rect::new(xi, yi, FIELD_CELL_SIZE, FIELD_CELL_SIZE);
                let color = field_to_color(self.0[x][y].position);

                graphics::set_color(ctx, color);
                graphics::rectangle(ctx, graphics::DrawMode::Fill, r)?;
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
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                // Decay intensity.
                let val = self.0[x][y].position * 0.95;
                self.0[x][y].position = val;
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
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                let mut val = self.0[x][y];
                let ix = x as i32;
                let iy = y as i32;

                val.position += val.velocity * dt;
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
                let forces = val.restoring_force() + neighbor_force / 1.0;
                val.velocity += forces;
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
                self.0[x][y].position = force;
            }
        }
    }


    fn sprinkle_random_bits(&mut self) {
        let tx = rand::random::<usize>() % FIELD_WIDTH;
        let ty = rand::random::<usize>() % FIELD_HEIGHT;
        self.0[tx][ty].position = 1.0;
    }
}


// The ndarray crate would be nice here.
struct MainState {
    field: Field,
    ship: Ship,
}

impl MainState {
    fn new(ctx: &mut ggez::Context) -> Self {
        let f = Field::new();
        MainState {
            field: f,
            ship: Ship::new(0 as i32, 0 as i32, ctx),
        }
    }

    fn draw_ship(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        self.ship.draw(ctx);

        Ok(())
    }
}

impl game::EventHandler for MainState {
    fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<()> {
        self.field.update();
        // println!("FPS: {}", ggez::timer::get_fps(ctx));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        graphics::clear(ctx);


        self.field.draw(ctx)?;
        self.draw_ship(ctx)?;

        ctx.renderer.present();
        Ok(())
    }

    fn key_down_event(&mut self, _keycode: Keycode, _keymod: Mod, _repeat: bool) {
        println!("Hi");
        self.ship.key_down_event(_keycode, _keymod, _repeat);
    }
}

fn default_conf() -> conf::Conf {
    let mut c = conf::Conf::new();
    c.window_title = String::from("wave-motion-gun");
    // c.window_width
    c
}

fn main() {
    let c = default_conf();
    let mut ctx = ggez::Context::load_from_conf("wave-motion-gun", c).unwrap();
    let state = MainState::new(&mut ctx);
    let mut g = game::Game::from_state(ctx, state);
    g.run().unwrap();
}
