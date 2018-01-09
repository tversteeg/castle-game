extern crate image;
extern crate blit;

use std::fs;

use blit::*;

fn save_blit_buffer_from_image(name: &str, mask_color: u32) {
    let path = format!("assets/{}", name);

    println!("Converting image \"{}\" to blit buffer", path);

    let img = image::open(path).unwrap();
    let img_as_rgb8 = match img.as_rgb8() {
        Some(i) => i,
        None => panic!("Could not convert image to RGB8 format")
    };

    let blit_buf = img_as_rgb8.as_blit_buffer(mask_color);

    blit_buf.save(format!("resources/{}.blit", name)).unwrap();
}

fn main() {
    fs::create_dir_all("resources").unwrap();

    let asset_paths = fs::read_dir("assets").unwrap();

    for path in asset_paths {
        let filepath = path.unwrap().path();
        let filename = filepath.file_name().unwrap();
        save_blit_buffer_from_image(filename.to_str().unwrap(), 0xFFFF00FF);
    }
}
