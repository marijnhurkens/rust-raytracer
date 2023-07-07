use ggez::graphics::Image;
use image::{ImageBuffer, Pixel, Rgb, RgbImage};
use nalgebra::Point2;

#[derive(Debug)]
pub enum ImageWrapMethod {
    Repeat,
    Black,
    Clamp,
}

#[derive(Debug)]
pub struct MipMap {
    image: RgbImage,
    wrap_method: ImageWrapMethod,
}

impl MipMap {
    pub fn new(image: RgbImage) -> Self {
        Self {
            image,
            wrap_method: ImageWrapMethod::Black,
        }
    }

    pub fn lookup(&self, point: Point2<f64>, width: f64) -> Rgb<f64> {
        let x = (self.image.dimensions().0 as f64 * point.x * 0.99) as u32;
        let y = (self.image.dimensions().1 as f64 * point.y * 0.99) as u32;
        let channels: Vec<f64> = self
            .image
            .get_pixel(x, y)
            .channels()
            .iter()
            .map(|x| *x as f64 / 255.0)
            .collect();

        Rgb(channels.try_into().unwrap())
    }
}
