use blit::*;
use git2::Repository;
use std::env;
use std::fs;

fn get_blit_buffer(path: &str, mask_color: u32) -> Option<BlitBuffer> {
    let img = image::open(path).unwrap();
    Some(blit_buffer(&img, Color::from_u32(mask_color)))
}

fn save_blit_buffer_from_image(folder: &str, name: &str, output: &str, mask_color: u32) {
    let path = format!("assets/{}/{}", folder, name);

    let blit_buf = get_blit_buffer(&path, mask_color).unwrap();
    blit_buf
        .save(format!(
            "{}/{}/{}.blit",
            env::var("OUT_DIR").unwrap(),
            folder,
            output
        ))
        .unwrap();
}

fn save_anim_buffer(folder: &str, name: &str, output: &str, mask_color: u32) {
    let path = format!("assets/{}/{}", folder, name);

    // Open the spritesheet info
    let file = fs::File::open(path).unwrap();
    let info: aseprite::SpritesheetData = serde_json::from_reader(file).unwrap();

    let blit_buf = {
        let image = info.meta.image.as_ref();

        get_blit_buffer(&image.unwrap(), mask_color).unwrap()
    };
    let anim_buffer = AnimationBlitBuffer::new(blit_buf, info);
    anim_buffer
        .save(format!(
            "{}/{}/{}.anim",
            env::var("OUT_DIR").unwrap(),
            folder,
            output
        ))
        .unwrap();
}

fn parse_folder(folder: &str, mask_color: u32) {
    fs::create_dir_all(format!("{}/{}", env::var("OUT_DIR").unwrap(), folder)).unwrap();

    let asset_paths = fs::read_dir(format!("assets/{}", folder)).unwrap();

    for path in asset_paths {
        let filepath = path.unwrap().path();
        let filename = filepath.file_name().unwrap();
        let filestem = filepath.file_stem().unwrap();
        let extension = filepath.extension().unwrap();

        // Rerun the build script if any of the assets changed
        println!("cargo:rerun-if-changed={:?}", filepath);

        match extension.to_str().unwrap() {
            "png" => save_blit_buffer_from_image(
                folder,
                filename.to_str().unwrap(),
                filestem.to_str().unwrap(),
                mask_color,
            ),
            "json" => save_anim_buffer(
                folder,
                filename.to_str().unwrap(),
                filestem.to_str().unwrap(),
                mask_color,
            ),
            other => panic!("Filetype not recognized: {}", other),
        }
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
