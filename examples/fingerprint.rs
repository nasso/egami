fn main() -> Result<(), image::ImageError> {
    let img = image::open("examples/red-panda.jpg")?;

    egami::fingerprint(&img, 64)
        .save("examples/thumbnail.png")
        .unwrap();

    Ok(())
}
