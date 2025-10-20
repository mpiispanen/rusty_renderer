use rusty_renderer::resources::TextureLoader;

fn main() {
    // Create checkerboard
    let checkerboard = TextureLoader::create_checkerboard(256, 32);
    let img =
        image::RgbaImage::from_raw(checkerboard.width, checkerboard.height, checkerboard.data)
            .unwrap();
    img.save("assets/textures/test_checkerboard.png").unwrap();
    println!("✓ Created assets/textures/test_checkerboard.png");

    // Create gradient
    let gradient = TextureLoader::create_gradient(256, 256);
    let img = image::RgbaImage::from_raw(gradient.width, gradient.height, gradient.data).unwrap();
    img.save("assets/textures/test_gradient.png").unwrap();
    println!("✓ Created assets/textures/test_gradient.png");
}
