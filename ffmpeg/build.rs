use std::*;
fn main() {
    println!("cargo:rustc-link-lib=avformat");
    println!("cargo:rustc-link-lib=swresample");
    println!("cargo:rustc-link-lib=swscale");
    println!("cargo:rustc-link-lib=avcodec");
    println!("cargo:rustc-link-lib=avutil");

    // If libraries are in non-standard locations
    // println!("cargo:rustc-link-search=/path/to/ffmpeg/libs");
    // Generate bindings

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .blocklist_var("FP_NORMAL")
        .blocklist_var("FP_NAN")
        .blocklist_var("FP_INFINITE")
        .blocklist_var("FP_ZERO")
        .blocklist_var("FP_SUBNORMAL")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = path::PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
