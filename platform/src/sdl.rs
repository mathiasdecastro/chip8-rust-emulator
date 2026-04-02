use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::FullscreenType;

pub struct SdlPlatform {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    event_pump: sdl2::EventPump,
    scale: u32,
    keys: [bool; 16],
}

impl SdlPlatform {
    pub fn new(scale: u32) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video = sdl_context.video().unwrap();

        let window = video
            .window("CHIP-8", 64 * scale, 32 * scale)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        canvas
            .window_mut()
            .set_fullscreen(FullscreenType::Desktop)
            .unwrap();

        Self {
            canvas,
            event_pump,
            scale,
            keys: [false; 16],
        }
    }

    pub fn draw(&mut self, pixels: &[[bool; 64]; 32]) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();

        self.canvas.set_draw_color(Color::WHITE);

        for (y, row) in pixels.iter().enumerate() {
            for (x, &pixel) in row.iter().enumerate() {
                if pixel {
                    let _ = self.canvas.fill_rect(Rect::new(
                        (x as u32 * self.scale) as i32,
                        (y as u32 * self.scale) as i32,
                        self.scale,
                        self.scale,
                    ));
                }
            }
        }

        self.canvas.present();
    }

    pub fn get_keys(&mut self) -> [bool; 16] {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => std::process::exit(0),

                Event::KeyDown {
                    keycode: Some(k), ..
                } => {
                    if let Some(i) = Self::map_key(k) {
                        self.keys[i] = true;
                    }
                }

                Event::KeyUp {
                    keycode: Some(k), ..
                } => {
                    if let Some(i) = Self::map_key(k) {
                        self.keys[i] = false;
                    }
                }

                _ => {}
            }
        }

        self.keys
    }

    pub fn map_key(key: Keycode) -> Option<usize> {
        match key {
            Keycode::Num1 => Some(0x1),
            Keycode::Num2 => Some(0x2),
            Keycode::Num3 => Some(0x3),
            Keycode::Num4 => Some(0xC),

            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::R => Some(0xD),

            Keycode::A => Some(0x7),
            Keycode::S => Some(0x8),
            Keycode::D => Some(0x9),
            Keycode::F => Some(0xE),

            Keycode::Z => Some(0xA),
            Keycode::X => Some(0x0),
            Keycode::C => Some(0xB),
            Keycode::V => Some(0xF),

            _ => None,
        }
    }
}
