use image;
use std::thread;
use std::time::Duration;
use std::cmp;
use IMAGE_BUFFER;
use camera::Camera;


const THREAD_COUNT: u32 = 16;


pub fn render(camera: Camera) {
    let image_width = IMAGE_BUFFER.read().unwrap().width();
    let image_height = IMAGE_BUFFER.read().unwrap().height();
    let work = image_width * image_height;
    let work_per_thread = work / THREAD_COUNT;

    println!("Start render, w{}, h{}", image_width, image_height);
    println!("Camera {:?}", camera);

    // thread id is used to divide the work
    for id in 0..THREAD_COUNT {
        let _thread = thread::spawn(move || {
            let thread_id = id.clone();
            let work_start = thread_id * work_per_thread;
            // prevent rounding error, cap at max work
            let work_end = cmp::min(work_start + work_per_thread, work);
         
            loop {
                for pos in work_start..work_end {
                    IMAGE_BUFFER.write().unwrap().put_pixel(
                        pos % image_width,
                        (pos / image_width) % image_height,
                        image::Rgba([
                            (20 * thread_id) as u8 % 255,
                            (20 * thread_id) as u8 % 255,
                            100,
                            255,
                        ]),
                    );

                    thread::sleep(Duration::from_millis( 1));
                }
            }
        });
    }
}
