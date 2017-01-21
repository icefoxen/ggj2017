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

use std::f64::consts;
use std::time::Duration;
use std::collections::HashSet;

fn magnitude(vec: &Vector2<f32>) -> f32 {
    (vec.x.powi(2) + vec.y.powi(2)).sqrt()
}

pub struct Ship {
    pub location: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub image: Image,

    scale: f32,
    bearing: Rotation2<f32>,
    speed: f64,
    turning_speed: f32,

    keys_down: HashSet<Keycode>
}

impl Ship {
    pub fn new(start_x: i32, start_y: i32, ctx: &mut Context) -> Self {
        Ship {
            location: Vector2::new(start_x as f32, start_y as f32),
            velocity: Vector2::new(0.0, 0.0),
            scale: 1.0 as f32,
            image: Image::new(ctx, "ship.png").unwrap(),
            speed: 1.0,
            turning_speed: 6.0,
            bearing: Rotation2::new(Vector1::new(0.0)),
            keys_down: HashSet::new()
        }
    }

    pub fn update(&mut self) {
        let speed = self.speed;
        let bearing = self.bearing;
        let velocity = self.velocity;

        self.location += velocity * speed as f32;

        self.velocity *= 0.9;

        for keycode in &self.keys_down {
            match *keycode {
                Keycode::W | Keycode::Up => {
                    let mag = magnitude(&velocity) + 1.0;
                    self.velocity = Vector2::new(mag * bearing.rotation().x.sin(), mag * -1.0 * bearing.rotation().x.cos());
                },
                Keycode::A | Keycode::Left => {
                    self.bearing.append_rotation_mut(&Vector1::new(-0.1));
                    let mag = magnitude(&velocity);
                    self.velocity = Vector2::new(mag * -1.0 * bearing.rotation().x.sin(), mag * -1.0 * bearing.rotation().x.cos());

                },
                Keycode::S | Keycode::Down => (),
                Keycode::D | Keycode::Right => {
                    self.bearing.append_rotation_mut(&Vector1::new(0.1));
                    let mag = magnitude(&velocity);
                    self.velocity = Vector2::new(mag * bearing.rotation().x.sin(), mag * bearing.rotation().x.cos());
                },
                _ => ()
            }
        }
        println!("bearing: {:?} velocity: {:?}", bearing.rotation().x.to_degrees(), velocity);
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        let r = graphics::Rect::new(self.location.x as i32, self.location.y as i32, (128.0 * self.scale) as u32, (128.0 * self.scale) as u32);

        self.image.draw_ex(ctx, None, Some(r), self.bearing.rotation().x.to_degrees() as f64, None, false, false);
    }

    pub fn key_down_event(&mut self, _keycode: Keycode) {
        self.keys_down.insert(_keycode);
    }

    pub fn key_up_event(&mut self, keycode: Keycode) {
        self.keys_down.remove(&keycode);
    }
}
