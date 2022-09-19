use image::ImageBuffer;

use crate::Film;

pub fn denoise(film: &mut Film) -> &mut Film {
    let image_width = film.image_size.x;
    let image_height = film.image_size.y;

    let mut normal_map = vec![0f32; image_width as usize * image_height as usize * 3];
    let mut albedo_map = vec![0f32; image_width as usize * image_height as usize * 3];
    film.pixels
        .clone()
        .iter()
        .enumerate()
        .for_each(|(i, pixel)| {
            normal_map[i * 3] = pixel.normal.x as f32;
            normal_map[i * 3 + 1] = pixel.normal.y as f32;
            normal_map[i * 3 + 2] = pixel.normal.z as f32;

            albedo_map[i * 3] = pixel.albedo.x as f32;
            albedo_map[i * 3 + 1] = pixel.albedo.y as f32;
            albedo_map[i * 3 + 2] = pixel.albedo.z as f32;
        });

    let temp = film.image_buffer.clone();
    let input_img: Vec<f32> = temp
        .into_raw()
        .iter()
        .map(|val| (*val as f32) / 255.0)
        .collect();
    let mut filter_output = vec![0.0f32; input_img.len()];

    let device = oidn::Device::new();

    oidn::RayTracing::new(&device)
        .srgb(true)
        .albedo_normal(&albedo_map[..], &normal_map[..])
        .clean_aux(true)
        .image_dimensions(image_width as usize, image_height as usize)
        .filter(&input_img[..], &mut filter_output[..])
        .expect("Filter config error!");

    if let Err(e) = device.get_error() {
        println!("Error denoising image: {}", e.1);
    }

    film.image_buffer = ImageBuffer::from_raw(
        image_width,
        image_height,
        filter_output
            .iter_mut()
            .map(|i| (*i * 255.0) as u8)
            .collect(),
    )
    .unwrap();

    film
}
