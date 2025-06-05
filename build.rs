use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Only build icons on macOS
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "macos" {
        println!("cargo:rerun-if-changed=resources/icons/");
        
        let out_dir = env::var("OUT_DIR").unwrap();
        let icons_dir = Path::new("resources/icons");
        
        if icons_dir.exists() {
            // Create icns file from PNG images
            create_icns_file(&out_dir);
        }
    }
}

fn create_icns_file(out_dir: &str) {
    use icns::{IconFamily, Image};
    use std::fs::File;
    use std::io::Cursor;
    
    let mut icon_family = IconFamily::new();
    
    // Define the files to include
    let files = [
        "icon-16.png",
        "icon-32.png", 
        "icon-64.png",
        "icon-128.png",
        "icon-256.png",
        "icon-512.png",
    ];
    
    for file_name in &files {
        let png_path = format!("resources/icons/{}", file_name);
        if let Ok(png_data) = fs::read(&png_path) {
            let cursor = Cursor::new(&png_data);
            if let Ok(image) = Image::read_png(cursor) {
                let _ = icon_family.add_icon(&image);
                println!("Added {} to icon family", file_name);
            }
        }
    }
    
    // Write the icns file
    let icns_path = format!("{}/icon.icns", out_dir);
    if let Ok(mut file) = File::create(&icns_path) {
        let _ = icon_family.write(&mut file);
        println!("Created icon.icns at {}", icns_path);
    }
}