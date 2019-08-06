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

    let hists = images
        .iter()
        .inspect(|_| {
            print!("Computing histogram... ");
            std::io::stdout().flush().unwrap();
        })
        // compute histogram
        .map(|img| egami::histogram(img, 64))
        .inspect(|_| println!("done."))
        .collect::<Vec<_>>();

    for i in 0..3 {
        for j in (i + 1)..3 {
            let hist_i = &hists[i];
            let hist_j = &hists[j];

            let similarity = hist_i.similarity(hist_j);

            println!(
                "Image #{} and #{} are {}% similar (r: {}%, g: {}%, b: {}%)",
                i,
                j,
                (similarity.average * 100.0),
                (similarity.channels[0] * 100.0),
                (similarity.channels[1] * 100.0),
                (similarity.channels[2] * 100.0),
            );
        }
    }

    Ok(())
}
