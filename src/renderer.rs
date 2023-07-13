use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::{EventPump, Sdl};

pub const GRID_X_SIZE: u32 = 64;
pub const GRID_Y_SIZE: u32 = 32;
pub const DOT_SIZE_IN_PXS: u32 = 10;

pub struct Renderer {
    canvas: WindowCanvas,
    sdl_context: Sdl,
    pub screen: [[u8; 64]; 32],
}

impl Renderer {
    pub fn new(sdl_context: &Sdl) -> Result<Renderer, String> {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(
                "chip-8",
                GRID_X_SIZE * DOT_SIZE_IN_PXS,
                GRID_Y_SIZE * DOT_SIZE_IN_PXS,
            )
            .position_centered()
            .opengl()
            .build()
            .expect("unable to create window");
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        Ok(Renderer {
            canvas,
            sdl_context: sdl_context.clone(),
            screen: [[0; GRID_X_SIZE as usize]; GRID_Y_SIZE as usize],
        })
    }

    pub fn event_pump(&self) -> EventPump {
        self.sdl_context.event_pump().unwrap()
    }

    fn draw_pixel(&mut self, x: u32, y: u32) {
        self.canvas.set_draw_color(Color::RGB(253, 195, 10));
        let _ = self.canvas.fill_rect(Rect::new(
            (x * DOT_SIZE_IN_PXS) as i32,
            (y * DOT_SIZE_IN_PXS) as i32,
            DOT_SIZE_IN_PXS,
            DOT_SIZE_IN_PXS,
        ));
    }

    fn draw_background(&mut self) {
        self.canvas.set_draw_color(Color::RGB(134, 84, 3));
        self.canvas.clear()
    }

    pub fn draw_screen(&mut self, screen: [[u8; 64]; 32]) {
        self.draw_background();
        screen.iter().enumerate().for_each(|(y, row)| {
            row.iter().enumerate().for_each(|(x, &pixel)| {
                if pixel != 0 {
                    self.draw_pixel(x.try_into().unwrap(), y.try_into().unwrap());
                }
            })
        });
        self.canvas.present();
    }
}
