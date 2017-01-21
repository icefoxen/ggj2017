extern crate ggez;
extern crate rand;

// extern crate ggez_goodies;

use ggez::GameResult;
use ggez::conf;
use ggez::game;
use ggez::graphics;
use ggez::graphics::Color;
use ggez::graphics::Drawable;

use std::time::Duration;
use std::boxed::Box;

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
    let negative_max = Color::RGBA(0, 0, 0, 255);
    let positive_max = Color::RGBA(0, 0, 128, 255);

    interp_between(val as f64, negative_max, positive_max)
}

type WaveType = f32;

// The ndarray crate would be nice here.
struct MainState {
    field: Vec<Vec<WaveType>>,
    ship: Ship
}

impl MainState {
    fn new(ctx: &mut ggez::Context) -> Self {
        let mut field = Vec::with_capacity(FIELD_WIDTH);
        for i in 0..FIELD_WIDTH {
            let mut bit = Vec::with_capacity(FIELD_HEIGHT);
            bit.resize(FIELD_HEIGHT, 0.5);
            field.push(bit);
        }
        MainState { field: field, ship: Ship::new(0 as i32, 0 as i32, ctx) }
    }

    fn draw_field(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                let xi = x as i32 * FIELD_CELL_SIZE as i32;
                let yi = y as i32 * FIELD_CELL_SIZE as i32;
                let r = graphics::Rect::new(xi, yi, FIELD_CELL_SIZE, FIELD_CELL_SIZE);

                let color = field_to_color(self.field[x][y]);
                graphics::set_color(ctx, color);
                graphics::rectangle(ctx, graphics::DrawMode::Fill, r)?;
            }
        }
        Ok(())
    }

    fn draw_ship(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {

        self.ship.draw(ctx);

        Ok(())
    }

    fn update_field(&mut self) {
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                // Decay intensity.
                let val = self.field[x][y] * 0.99;
                self.field[x][y] = val;
            }
        }
        self.sprinkle_random_bits();
    }

    fn sprinkle_random_bits(&mut self) {
        let tx = rand::random::<usize>() % FIELD_WIDTH;
        let ty = rand::random::<usize>() % FIELD_HEIGHT;
        self.field[tx][ty] = 1.0;
    }
}

impl game::EventHandler for MainState {
    fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<()> {
        self.update_field();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        graphics::clear(ctx);

        self.draw_field(ctx);
        self.draw_ship(ctx);

        ctx.renderer.present();
        Ok(())
    }
}

fn default_conf() -> conf::Conf {
    let mut c = conf::Conf::new();
    // c.window_width
    c
}

fn main() {
    let c = default_conf();
    let mut ctx = ggez::Context::load_from_conf("wave-motion-gun", c).unwrap();
    let state = MainState::new(&mut ctx);
    let mut g = game::Game::from_state(ctx, state);
    g.run();
}
