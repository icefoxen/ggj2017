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
const SHIP_SIZE: f32 = 128.0;

// Redundant declaration, any way to use constants in main.rs?
const WINDOW_HEIGHT: usize = 600;
const WINDOW_WIDTH: usize = 800;
// Again, redundant
fn clamp(val: f32, lower: f32, upper: f32) -> f32 {
    f32::min(f32::max(val, lower), upper)
}

fn magnitude(vec: &Vector2<f32>) -> f32 {
    (vec.x.powi(2) + vec.y.powi(2)).sqrt()
}

fn calculate_jump_scale(index: usize) -> f32 {
    let data = vec![1.0, 0.95, 1.0, 1.2, 1.5, 1.8, 2.0, 1.8, 1.5, 1.2, 1.0];
    data[index / 6] * 0.75
}

pub struct Ship {
    pub location: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub angular_velocity: f32,
    pub image: Image,

    scale: f32,
    bearing: f32,
    speed: f32,
    turning_torque: f32,
    length: f32,
    width: f32,
    collider_radius: f32,
    jump_index: usize,
    pub jumping: bool,
    pub post_jump: usize,

    keys_down: HashSet<Buttons>,
}

impl Ship {
    pub fn new(start_x: i32, start_y: i32, ctx: &mut Context, filename: &str) -> Self {
        Ship {
            location: Vector2::new(start_x as f32, start_y as f32),
            velocity: Vector2::new(0.0, 0.0),
            angular_velocity: 0.0,
            scale: 1.0,
            image: Image::new(ctx, filename).unwrap(),
            speed: 0.2,
            turning_torque: 0.001,
            bearing: 0.0,
            length: 128.0,
            width: 128.0,
            collider_radius: 64.0 * 1.414,
            jumping: false,
            jump_index: 0,
            post_jump: 0,

            keys_down: HashSet::new(),
        }
    }

    pub fn jump(&mut self) {
        if !self.jumping && self.post_jump == 0 {
            println!("Jumping starting?");
            self.jumping = true;
            self.jump_index = 0;
        }
    }

    pub fn update(&mut self) {
        let speed = self.speed;
        let velocity = self.velocity;
        let mut acceleration: Vector2<f32> = na::zero();
        let mut torque: f32 = 0.0;
        // let center: Vector2<f32> = self.location +
        //                            Vector2::new(0.5 * self.width, 0.5 * self.length);

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
        //
        // Trying to add torque

        for keycode in &self.keys_down {
            match *keycode {
                Buttons::Up => {
                    let facing_vec_x = f32::cos(self.bearing - consts::PI / 2.0);
                    let facing_vec_y = f32::sin(self.bearing - consts::PI / 2.0);
                    let force = Vector2::new(facing_vec_x, facing_vec_y);
                    acceleration += force;
                }
                Buttons::Left => {
                    // self.bearing -= self.turning_speed;
                    torque -= self.turning_torque;
                }
                Buttons::Right => {
                    // self.bearing += self.turning_speed;
                    torque += self.turning_torque;
                }
            }
        }


        // Cooldown timer that makes you invincible after jumps
        // but maybe also prevents you from jumping constantly
        if self.post_jump > 0 {
            self.post_jump -= 1;
        }
        if self.jumping {
            // println!("Doing jump at index {}, scale is {}",
            //          self.jump_index,
            //          self.scale);
            self.scale = calculate_jump_scale(self.jump_index);
            self.jump_index += 1;

            if self.jump_index == 61 {
                // println!("Done jumping");
                self.scale = 1.0;
                self.jump_index = 0;
                self.jumping = false;
                self.post_jump = 30;
            }
        }

        self.velocity += acceleration;
        self.velocity *= DRAG;
        self.location += velocity * speed as f32;
        self.location.x = clamp(self.location.x,
                                self.collider_radius,
                                WINDOW_WIDTH as f32 - self.collider_radius);
        self.location.y = clamp(self.location.y,
                                self.collider_radius,
                                WINDOW_HEIGHT as f32 - self.collider_radius);


        self.angular_velocity += torque;
        self.bearing += self.angular_velocity;
        self.angular_velocity *= DRAG;

        // println!("bearing: {:?} velocity: {:?}", self.bearing, velocity);
        // println!("location: {:?}, {:?}", self.location.x, self.location.y);
        // println!("center: {:?}, {:?}, radius: {:?}",
        //         center.x,
        //         center.y,
        //         self.collider_radius);
    }


    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let size = 128.0 * self.scale;
        let half_size = size / 2.0;
        let r = graphics::Rect::new((self.location.x - half_size) as i32,
                                    (self.location.y - half_size) as i32,
                                    size as u32,
                                    size as u32);
        // let c = graphics::Point::new((0.0 * self.scale) as i32,
        //                             (0.0 * self.scale) as i32);

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
