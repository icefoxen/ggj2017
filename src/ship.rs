use ggez;
use ggez::game;
use ggez::event::*;
use ggez::Context;
use ggez::graphics;
use ggez::GameResult;
use ggez::graphics::Image;
use ggez::graphics::Drawable;

use na;
use na::Vector2;

use std::f32::consts;
use std::collections::HashSet;

const DRAG: f32 = 0.97;
const RAD_TO_DEGREES: f32 = 180.0 / consts::PI;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Buttons {
    Up,
    Left,
    Right,
}

fn magnitude(vec: &Vector2<f32>) -> f32 {
    (vec.x.powi(2) + vec.y.powi(2)).sqrt()
}

pub struct Ship {
    pub location: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub image: Image,

    scale: f32,
    bearing: f32,
    speed: f32,
    keel_strength: f32,
    turning_speed: f32,

    keys_down: HashSet<Buttons>,
}

impl Ship {
    pub fn new(start_x: i32, start_y: i32, ctx: &mut Context) -> Self {
        Ship {
            location: Vector2::new(start_x as f32, start_y as f32),
            velocity: Vector2::new(0.0, 0.0),
            scale: 1.0,
            image: Image::new(ctx, "ship.png").unwrap(),
            speed: 0.2,
            keel_strength: 0.1,
            turning_speed: 0.03,
            bearing: 0.0,
            keys_down: HashSet::new(),
        }
    }

    pub fn update(&mut self) {
        let speed = self.speed;
        let velocity = self.velocity;
        let mut acceleration: Vector2<f32> = na::zero();

        // We want you to have a bit of velocity based
        // on your bearing but not too much, so you can
        // power slide a bit but still have some keel.
        // We'll try doing real-ish physics and just exert a
        // force on your ship perpendicular to your facing.
        //
        // oooooh it has to be scaled by your velocity vector,
        // but not the whole thing but the component of it that
        // is in the direction of your facing! ...or something.
        //
        // I can't brain this right now, I'm leaving it for the moment.
        // let facing_vec_x = f32::cos(self.bearing);
        // let facing_vec_y = f32::sin(self.bearing);
        // acceleration += Vector2::new(facing_vec_x, facing_vec_y) * self.keel_strength;

        for keycode in &self.keys_down {
            match *keycode {
                Buttons::Up => {
                    let facing_vec_x = f32::cos(self.bearing - consts::PI / 2.0);
                    let facing_vec_y = f32::sin(self.bearing - consts::PI / 2.0);
                    let force = Vector2::new(facing_vec_x, facing_vec_y);
                    acceleration += force;
                }
                Buttons::Left => {
                    self.bearing -= self.turning_speed;

                }
                Buttons::Right => {
                    self.bearing += self.turning_speed;
                }
            }
        }

        self.velocity += acceleration;
        self.velocity *= DRAG;
        self.location += velocity * speed as f32;
        // println!("bearing: {:?} velocity: {:?}", bearing, velocity);
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let size = 128.0;
        let half_size = 64;
        let r = graphics::Rect::new(self.location.x as i32 - half_size,
                                    self.location.y as i32 - half_size,
                                    (size * self.scale) as u32,
                                    (size * self.scale) as u32);
        self.image
            .draw_ex(ctx,
                     None,
                     Some(r),
                     (self.bearing * RAD_TO_DEGREES) as f64,
                     None,
                     false,
                     false)
    }

    pub fn key_down_event(&mut self, button: Buttons) {
        self.keys_down.insert(button);
    }

    pub fn key_up_event(&mut self, button: Buttons) {
        self.keys_down.remove(&button);
    }
}
