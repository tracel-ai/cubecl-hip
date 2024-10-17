fn main() {
    println!("cargo:rustc-link-lib=dylib=hiprtc");
    println!("cargo:rustc-link-lib=dylib=amdhip64");
    println!("cargo:rustc-link-search=native=/opt/rocm/lib");
}
