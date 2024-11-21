use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CUBECL_ROCM_PATH");
    println!("cargo:rerun-if-env-changed=ROCM_PATH");
    println!("cargo:rerun-if-env-changed=HIP_PATH");

    let mut paths: Vec<_> = ["CUBECL_ROCM_PATH", "ROCM_PATH", "HIP_PATH"]
        .into_iter()
        .filter_map(|var| env::var(var).ok())
        .collect();
    // default installation path
    paths.push("/opt/rocm".to_string());

    let rocm_path = paths.iter().find(|path| {
        let hip_path = Path::new(path).join("include/hip");
        hip_path.exists()
    });

    if let Some(valid_rocm_path) = rocm_path {
        println!("cargo:rustc-link-lib=dylib=hiprtc");
        println!("cargo:rustc-link-lib=dylib=amdhip64");
        println!("cargo:rustc-link-search=native={}/lib", valid_rocm_path);
    } else if paths.len() > 1 {
        panic!("HIP headers not found in any of the defined CUBECL_ROCM_PATH, ROCM_PATH or HIP_PATH directories.");
    }
}
