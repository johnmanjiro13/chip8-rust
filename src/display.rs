use iced::canvas::{Canvas, Cursor, Frame, Geometry, Path, Program};
use iced::{Color, Element, Length, Point, Rectangle, Size};

pub const WIDTH: usize = DISPLAY_WIDTH * PIXEL_SIZE;
pub const HEIGHT: usize = DISPLAY_HEIGHT * PIXEL_SIZE;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const PIXEL_SIZE: usize = 10;

pub struct Display {
    pixels: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
}

impl Display {
    pub fn new() -> Self {
        Self {
            pixels: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
        }
    }

    pub fn view(&mut self) -> Element<()> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Program<()> for Display {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let pixel_size = Size::new(PIXEL_SIZE as f32, PIXEL_SIZE as f32);

        let mut frame = Frame::new(bounds.size());
        let background = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&background, Color::BLACK);

        let pixels = Path::new(|p| {
            for y in 0..DISPLAY_HEIGHT {
                for x in 0..DISPLAY_WIDTH {
                    if self.pixels[y][x] {
                        p.rectangle(
                            Point::new((x * PIXEL_SIZE) as f32, (y * PIXEL_SIZE) as f32),
                            pixel_size,
                        )
                    }
                }
            }
        });
        frame.fill(&pixels, Color::WHITE);
        vec![frame.into_geometry()]
    }
}
