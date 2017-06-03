extern crate bindgen;
extern crate metadeps;

use std::fs::OpenOptions;
use std::io::Write;

fn format_write(builder: bindgen::Builder, output: &str) {
    let s = builder.generate()
        .unwrap()
        .to_string()
        .replace("/**", "/*")
        .replace("/*!", "/*");

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output)
        .unwrap();

    let _ = file.write(s.as_bytes());
}

fn common_builder() -> bindgen::Builder {
    bindgen::builder()
        .raw_line("#![allow(dead_code)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(non_upper_case_globals)]")
}

fn main() {
    let libs = metadeps::probe().unwrap();
    // TODO pass include paths to bindgen
    // let headers = libs.get("x264").unwrap().include_paths.clone();
    let buildver = libs.get("x264").unwrap().version.split(".").nth(1).unwrap();

    let builder = common_builder()
        .raw_line(format!("pub unsafe fn x264_encoder_open(params: *mut x264_param_t) -> *mut x264_t {{
                               x264_encoder_open_{}(params)
                          }}", buildver))
        .header("data/x264.h");

    // Manually fix the comment so rustdoc won't try to pick them
    format_write(builder, "src/x264.rs");
}
