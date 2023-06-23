use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;

pub const GRID_X_SIZE: u32 = 64;
pub const GRID_Y_SIZE: u32 = 32;
pub const DOT_SIZE_IN_PXS: u32 = 10;

pub struct Renderer {
    canvas: WindowCanvas,
    pub screen: [[u8; 64]; 32],
}

impl Renderer {
    pub fn new(window: Window) -> Result<Renderer, String> {
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        Ok(Renderer {
            canvas,
            screen: [[0; GRID_X_SIZE as usize]; GRID_Y_SIZE as usize],
        })
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

    pub fn draw_screen(&mut self) {
        self.draw_background();
        for row in 0..self.screen.len() {
            for col in 0..self.screen[row].len() {
                if self.screen[row][col] != 0 {
                    self.draw_pixel(col.try_into().unwrap(), row.try_into().unwrap());
                }
            }
        }
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn clear_screen(&mut self) {
        self.screen.iter_mut().for_each(|row| {
            row.iter_mut().for_each(|pixel| {
                *pixel = 0;
            })
        });
    }
}
