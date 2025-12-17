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
        let (w, h) = self.image.dimensions();

        // U: Repeat
        let u = point.x - point.x.floor();

        // V: Clamp
        let v = point.y.clamp(0.0, 1.0);

        let x = ((u * w as f64) as u32).min(w - 1);
        let y = ((v * h as f64) as u32).min(h - 1);

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
