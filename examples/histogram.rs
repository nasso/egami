use std::io::Write;

fn main() -> Result<(), image::ImageError> {
    print!("Loading images... ");
    std::io::stdout().flush().unwrap();

    let images = [
        image::open("examples/red-panda.jpg")?,
        image::open("examples/red-panda-small.jpg")?,
        image::open("examples/goat.jpg")?,
    ];

    println!("done.");

    for size in &[8, 64, 256] {
        let hists = images
            .iter()
            .inspect(|_| {
                print!("Computing histogram of size {}... ", size);
                std::io::stdout().flush().unwrap();
            })
            // compute histogram and maximize it
            .map(|img| egami::histogram(img, *size).maximized())
            .inspect(|_| println!("done."));

        // save each to an image file
        for (img, hist) in hists.enumerate() {
            let mut himg = image::ImageBuffer::new(256, 256);
            for (x, y, pxl) in himg.enumerate_pixels_mut() {
                let i = (x as f64 / 256.0 * *size as f64) as usize;

                *pxl = image::Rgb([
                    if hist.channels[0].data[i] > (1.0 - y as f64 / 256.0) {
                        255
                    } else {
                        0
                    },
                    if hist.channels[1].data[i] > (1.0 - y as f64 / 256.0) {
                        255
                    } else {
                        0
                    },
                    if hist.channels[2].data[i] > (1.0 - y as f64 / 256.0) {
                        255
                    } else {
                        0
                    },
                ])
            }

            himg.save(format!("examples/histogram_x{}-{}.png", size, img + 1))
                .unwrap();
        }
    }

    Ok(())
}
