use ggez;
use ggez::game;
use ggez::event::*;
use ggez::Context;
use ggez::graphics;
use ggez::GameResult;
use ggez::graphics::Image;
use ggez::graphics::Drawable;

use na;
use na::{Vector1, Vector2, Rotation2, Rotation, Rotate};

use std::f32::consts;
use std::time::Duration;
use std::collections::HashSet;

const DRAG: f32 = 0.99;
const RAD_TO_DEGREES: f32 = 180.0 / consts::PI;


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
    turning_speed: f32,

    keys_down: HashSet<Keycode>,
}

impl Ship {
    pub fn new(start_x: i32, start_y: i32, ctx: &mut Context) -> Self {
        Ship {
            location: Vector2::new(start_x as f32, start_y as f32),
            velocity: Vector2::new(0.0, 0.0),
            scale: 1.0,
            image: Image::new(ctx, "ship.png").unwrap(),
            speed: 0.2,
            turning_speed: 0.01,
            bearing: 0.0,
            keys_down: HashSet::new(),
        }
    }

    pub fn update(&mut self) {
        let speed = self.speed;
        let bearing = self.bearing;
        let velocity = self.velocity;

        self.location += velocity * speed as f32;

        self.velocity *= DRAG;

        for keycode in &self.keys_down {
            match *keycode {
                Keycode::W | Keycode::Up => {
                    let facing_vec_x = f32::cos(self.bearing - consts::PI / 2.0);
                    let facing_vec_y = f32::sin(self.bearing - consts::PI / 2.0);
                    let accel = Vector2::new(facing_vec_x, facing_vec_y) * self.speed;
                    self.velocity += accel;
                    // let mag = magnitude(&velocity) + 1.0;
                    // self.velocity = Vector2::new(mag * bearing.rotation().x.sin(),
                    //                              mag * -1.0 * bearing.rotation().x.cos());
                }
                Keycode::A | Keycode::Left => {
                    self.bearing -= self.turning_speed;
                    // self.bearing.append_rotation_mut(&Vector1::new(-0.1));
                    // let mag = magnitude(&velocity);
                    // self.velocity = Vector2::new(mag * -1.0 * bearing.rotation().x.sin(),
                    // mag * -1.0 * bearing.rotation().x.cos());

                }
                Keycode::S | Keycode::Down => (),
                Keycode::D | Keycode::Right => {
                    self.bearing += self.turning_speed;
                    // self.bearing.append_rotation_mut(&Vector1::new(0.1));
                    // let mag = magnitude(&velocity);
                    // self.velocity = Vector2::new(mag * bearing.rotation().x.sin(),
                    //                              mag * bearing.rotation().x.cos());
                }
                _ => (),
            }
        }
        println!("bearing: {:?} velocity: {:?}", bearing, velocity);
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        let r = graphics::Rect::new(self.location.x as i32,
                                    self.location.y as i32,
                                    (128.0 * self.scale) as u32,
                                    (128.0 * self.scale) as u32);

        self.image.draw_ex(ctx,
                           None,
                           Some(r),
                           (self.bearing * RAD_TO_DEGREES) as f64,
                           None,
                           false,
                           false);
    }

    pub fn key_down_event(&mut self, _keycode: Keycode) {
        self.keys_down.insert(_keycode);
    }

    pub fn key_up_event(&mut self, keycode: Keycode) {
        self.keys_down.remove(&keycode);
    }
}
