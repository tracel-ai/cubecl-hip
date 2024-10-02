extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // TODO create a feature for AMD and CUDA platforms
    //      for now we assume AMD platform
    println!("cargo:rustc-link-lib=dylib=hiprtc");
    println!("cargo:rustc-link-lib=dylib=amdhip64");
    println!("cargo:rustc-link-search=native=/opt/rocm/lib");

    let header = "wrapper.h";
    // Generate bindings using bindgen
    let bindings = bindgen::Builder::default()
        .header(header)
        .clang_arg("-D__HIP_PLATFORM_AMD__")
        .clang_arg("-I/opt/rocm/include")
        .generate()
        .expect("Should generate HIP RTC bindings");
    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Should write bindings to binding.rs file");
}
