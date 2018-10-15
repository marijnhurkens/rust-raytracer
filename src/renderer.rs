use image;
use std::thread;
use std::time::Duration;
use IMAGE_BUFFER;



pub fn render() {
    let image_width = IMAGE_BUFFER.read().unwrap().width();
    let image_height = IMAGE_BUFFER.read().unwrap().height();

    println!(
        "Test render, w{}, h{}",
        image_width,
        image_height
    );

    let thread = thread::spawn(move || {
        for x in 0..image_width {
            for y in 0..image_height {
                IMAGE_BUFFER.write().unwrap().put_pixel(x, y, image::Rgba([x as u8 % 255, y as u8 % 255, 0, 255]));
                //IMAGE_BUFFER.write().unwrap().put_pixel(x, y, image::Rgba([ 255, 255, 255, 255]);
      
                thread::sleep(Duration::from_millis(500));
                //println!("Write pixel, buff {:?}", IMAGE_BUFFER.read().unwrap());
            }
        }
    });
}
