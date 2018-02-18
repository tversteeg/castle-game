extern crate image;
extern crate blit;
extern crate git2;

use std::fs;

use blit::*;
use git2::Repository;

fn save_blit_buffer_from_image(folder: &str, name: &str, mask_color: u32) {
    let path = format!("assets/{}/{}", folder, name);

    println!("Converting image \"{}\" to blit buffer", path);

    let img = image::open(path).unwrap();
    let img_as_rgb8 = match img.as_rgb8() {
        Some(i) => i,
        None => panic!("Could not convert image to RGB8 format")
    };

    let blit_buf = img_as_rgb8.to_blit_buffer(Color::from_u32(mask_color));

    blit_buf.save(format!("resources/{}/{}.blit", folder, name)).unwrap();
}

fn parse_folder(folder: &str, mask_color: u32) {
    fs::create_dir_all(format!("resources/{}", folder)).unwrap();

    let asset_paths = fs::read_dir(format!("assets/{}", folder)).unwrap();

    for path in asset_paths {
        let filepath = path.unwrap().path();
        let filename = filepath.file_name().unwrap();
        save_blit_buffer_from_image(folder, filename.to_str().unwrap(), mask_color);
    }
}

fn main() {
    if !std::path::Path::new("assets").exists() {
        let url = "https://github.com/tversteeg/castle-game-assets.git";
        if let Err(e) = Repository::clone(url, "assets") {
            panic!("Failed to clone repository: {}", e);
        }
    }

    parse_folder("sprites", 0xFFFF00FF);
    parse_folder("masks", 0xFF000000);

    parse_folder("gui", 0xFFFF00FF);
}
