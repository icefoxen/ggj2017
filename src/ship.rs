use ggez;
use ggez::game;
use ggez::event::*;
use ggez::Context;
use ggez::graphics;
use ggez::GameResult;
use ggez::graphics::Image;
use ggez::graphics::Drawable;

use na::{Vector2, Rotation2};

use std::time::Duration;
use std::vec::Vec;

pub struct Ship {
    pub location: Vector2,
    pub image: Image,

    scale: f32,
    bearing: Rotation2(),
    speed: f64,
    turning_speed: f64,

    keys_down: Vec<Keycode>
}

impl Ship {
    pub fn new(start_x: i32, start_y: i32, ctx: &mut Context) -> Self {
        Ship {
            location: Vector2::new(start_x as f32, start_y as f32),
            scale: 1.0 as f32,
            image: Image::new(ctx, "ship.png").unwrap(),
            speed: 6.0,
            turning_speed: 3.0,
            bearing: 0.0
        }
    }

    pub fn update() {
        for keycode in keys_down {
            match keycode {
                Keycode::W | Keycode::Up => {
                    self.location = (x + (bearing.cos() * speed) as i32, y + (bearing.sin() * speed) as i32);
                },
                Keycode::A | Keycode::Left => {
                    self.bearing = bearing - self.turning_speed;
                },
                Keycode::S | Keycode::Down => {
                    self.location = (x, y);
                },
                Keycode::D | Keycode::Right => {
                    self.bearing = bearing + self.turning_speed;
                },
                _ => ()
            }
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        let r = graphics::Rect::new(self.location.0, self.location.1, (128.0 * self.scale) as u32, (128.0 * self.scale) as u32);

        self.image.draw_ex(ctx, None, Some(r), self.bearing, None, false, false);
    }

    pub fn key_down_event(&mut self, _keycode: Keycode, _keymod: Mod, _repeat: bool) {
        let speed = self.speed;
        let bearing = self.bearing;
        let (x,y) = self.location;

    }
}
