use ggez;
use ggez::game;
use ggez::event;
use ggez::Context;
use ggez::graphics;
use ggez::graphics::Image;
use ggez::graphics::Drawable;

pub struct Ship {
    pub location: (i32, i32),
    pub scale: f32,
    pub image: Image,
}

impl Ship {
    pub fn new(start_x: i32, start_y: i32, ctx: &mut Context) -> Self {
        Ship {
            location: (start_x, start_y),
            scale: 1.0 as f32,
            image: Image::new(ctx, "ship.png").unwrap()
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        let r = graphics::Rect::new(self.location.0, self.location.1, 128, 128);

        self.image.draw(ctx, None, Some(r));
    }
}

impl game::EventHandler for Ship {
    fn key_down_event(&mut self, _keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match _keycode {
            W => {
                self.location = (self.location.0, self.location.1 + 1);
            },
            A => {
                self.location = (self.location.0 - 1, self.location.1);
            },
            S => {
                self.location = (self.location.0, self.location.1 - 1);
            },
            D => {
                self.location = (self.location.0 + 1, self.location.1 - 1);
            },
        }
    }
}
