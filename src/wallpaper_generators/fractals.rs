use image::ImageBuffer;
use num_complex::Complex;
pub(crate) fn generate_julia_set(width: u32, height: u32) -> () {
    let scalex = 3.0 / width as f64;
    let scaley = 3.5 / height as f64;
    let mut imgbuf = ImageBuffer::new(width, height);

    // Initial background
    // TODO: Make this more interesting - maybe random colors?
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (0.3 * x as f64) as u8;
        let b = (0.3 * y as f64) as u8;
        *pixel = image::Rgb([r, 0, b]);
    }

    // Generate julia set
    // TODO: make more interesting by randomizing the c value
    for x in 0..width {
        for y in 0..height {
            let cx = x as f64 * scalex - 1.5;
            let cy = y as f64 * scaley - 1.5;

            let c = Complex::new(-0.4, 0.6);
            let mut z = Complex::new(cx, cy);

            let mut i = 0;
            while i < 255 && z.norm() <= 2.0 {
                z = z * z + c;
                i += 1;
            }

            let pixel = imgbuf.get_pixel_mut(x, y);
            let image::Rgb(data) = *pixel;
            *pixel = image::Rgb([data[0], i as u8, data[2]]);
        }
    }

    imgbuf.save("julia.png").unwrap();
}

pub(crate) fn generate_mandelbrot_set(width: u32, height: u32) -> () {
    todo!("Implement for generating mandelbrot set")
}