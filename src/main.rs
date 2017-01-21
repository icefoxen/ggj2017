extern crate ggez;
extern crate rand;

// extern crate ggez_goodies;

use ggez::GameResult;
use ggez::conf;
use ggez::game;
use ggez::graphics;
use ggez::graphics::Color;

use std::time::Duration;
use std::boxed::Box;

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
    let negative_max = Color::RGBA(0, 0, 0, 255);
    let positive_max = Color::RGBA(0, 0, 128, 255);

    interp_between(val as f64, negative_max, positive_max)
}

type WaveType = f32;

struct Field(Vec<Vec<f32>>);

impl Field {
    fn new() -> Self {
        let mut field = Vec::with_capacity(FIELD_WIDTH);
        for i in 0..FIELD_WIDTH {
            let mut bit = Vec::with_capacity(FIELD_HEIGHT);
            bit.resize(FIELD_HEIGHT, 0.5);
            field.push(bit);
        }

        let mut f = Field(field);
        f.initial_conditions();
        f
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                let xi = x as i32 * FIELD_CELL_SIZE as i32;
                let yi = y as i32 * FIELD_CELL_SIZE as i32;
                let r = graphics::Rect::new(xi, yi, FIELD_CELL_SIZE, FIELD_CELL_SIZE);
                let color = field_to_color(self.0[x][y]);
                graphics::set_color(ctx, color);
                graphics::rectangle(ctx, graphics::DrawMode::Fill, r)?;
            }
        }
        Ok(())
    }

    fn update(&mut self) {
        // self.sprinkle_random_bits();
    }

    // For now we copy off the matlab code at
    //
    // It's a one-dimensional simulation for now, let's
    // using the x axis as time and the y axis as space.
    fn initial_conditions(&mut self) {
        // Number of timesteps
        let t = FIELD_WIDTH as f32;
        // frequency of source
        let f = 100.0;
        // wave velocity
        let v = 100.0;
        // time step
        let dt = 0.01;
        // CFL condition, v * (dt/dx), but dx is 1 (one cell) so.
        let c = v * dt;
        let s1 = f32::floor(t / f);

        for i in 0..FIELD_HEIGHT {
            let t = i as f32 * dt;
            let v = f32::sin(2.0 * std::f32::consts::PI * f * dt * t);
            self.0[0][i] = v;
        }


        for i in 0..FIELD_HEIGHT {
            let t = i as f32 * dt;
            let v = f32::sin(2.0 * std::f32::consts::PI * f * dt * t);
            self.0[1][i] = v;
        }


        for j in 3..FIELD_WIDTH {
            for i in 2..FIELD_HEIGHT - 1 {
                let u1 = 2.0 * self.0[j - 1][i] - self.0[j - 2][i];
                let u2 = self.0[j - 1][i - 1] - 2.0 * self.0[j - 1][i + 1];
                self.0[j][i] = u1 + c * c * u2;
            }
        }
    }

    fn decay(&mut self) {
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                // Decay intensity.
                let val = self.0[x][y] * 0.99;
                self.0[x][y] = val;
            }
        }
    }

    fn propegate(&mut self) {
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {

            }
        }
    }


    fn sprinkle_random_bits(&mut self) {
        let tx = rand::random::<usize>() % FIELD_WIDTH;
        let ty = rand::random::<usize>() % FIELD_HEIGHT;
        self.0[tx][ty] = 1.0;
    }
}


// The ndarray crate would be nice here.
struct MainState {
    field: Field,
}

impl MainState {
    fn new() -> Self {
        let f = Field::new();
        MainState { field: f }
    }
}

impl game::EventHandler for MainState {
    fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<()> {
        self.field.update();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        graphics::clear(ctx);

        self.field.draw(ctx);
        ctx.renderer.present();
        Ok(())
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
    let state = MainState::new();
    let ctx = ggez::Context::load_from_conf("wave-motion-gun", c).unwrap();
    let mut g = game::Game::from_state(ctx, state);
    g.run();
}
