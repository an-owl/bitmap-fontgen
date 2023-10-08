use std::path::PathBuf;
use std::str::FromStr;

fn main() {
    let p = PathBuf::from_str("../../res/ter-u32n.bdf").unwrap().canonicalize().unwrap();
    let mut o = std::env::var("OUT_DIR").unwrap().to_string();
    o += "test_single_file.rs";
    fontgen::codegen::gen_font(vec![p.clone()],&mut std::fs::File::create(&o).unwrap());
    println!("cargo:rustc-env=FONT_FILE={}",o);
    println!("cargo:rustc-env=FONT_ORIGIN={}",p.display());
}