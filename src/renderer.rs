use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;

pub const GRID_X_SIZE: u32 = 64;
pub const GRID_Y_SIZE: u32 = 32;
pub const DOT_SIZE_IN_PXS: u32 = 10;

pub struct Renderer {
    canvas: WindowCanvas,
}

impl Renderer {
    pub fn new(window: Window) -> Result<Renderer, String> {
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        Ok(Renderer { canvas })
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32) {
        self.canvas.set_draw_color(Color::RGB(253, 195, 10));
        let _ = self.canvas.fill_rect(Rect::new(
            (x * DOT_SIZE_IN_PXS) as i32,
            (y * DOT_SIZE_IN_PXS) as i32,
            DOT_SIZE_IN_PXS,
            DOT_SIZE_IN_PXS,
        ));
    }

    pub fn draw_background(&mut self) {
        self.canvas.set_draw_color(Color::RGB(134, 84, 3));
        self.canvas.clear()
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }
}
