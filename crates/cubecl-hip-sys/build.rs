use std::env;
use std::path::Path;

fn main() {
    let env_vars = [
        "CUBECL_ROCM_PATH",
        "ROCM_PATH",
    ];

    let rocm_path = env_vars
        .into_iter()
        .map(env::var)
        .filter_map(Result::ok)
        .find(|path| {
            let hip_path = Path::new(path).join("include/hip");
            hip_path.exists()
        });

    if let Some(found_rocm_path) = rocm_path {
        println!("cargo:rustc-link-lib=dylib=hiprtc");
        println!("cargo:rustc-link-lib=dylib=amdhip64");
        println!("cargo:rustc-link-search=native={}/lib", found_rocm_path);
    } else {
        panic!("Please set the CUBECL_ROCM_PATH or ROCM_PATH environment variable to a valid directory containing the ROCm HIP installation.");
    }
}
