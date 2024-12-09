use std::env;
use std::path::Path;

/// Reads a header inside the rocm folder, that contains the lib's version
fn get_system_hip_version(rocm_path: impl AsRef<Path>) -> std::io::Result<(u8, u8, u32)> {
    let version_path = rocm_path.as_ref().join("include/hip/hip_version.h");
    let version_file = std::fs::read_to_string(version_path)?;
    let version_lines = version_file.lines().collect::<Vec<_>>();

    let system_major = version_lines
        .iter()
        .find_map(|line| line.strip_prefix("#define HIP_VERSION_MAJOR "))
        .expect("Invalid hip_version.h file structure: Major version line not found")
        .parse::<u8>()
        .expect("Invalid hip_version.h file structure: Couldn't parse major version");
    let system_minor = version_lines
        .iter()
        .find_map(|line| line.strip_prefix("#define HIP_VERSION_MINOR "))
        .expect("Invalid hip_version.h file structure: Minor version line not found")
        .parse::<u8>()
        .expect("Invalid hip_version.h file structure: Couldn't parse minor version");
    let system_patch = version_lines
        .iter()
        .find_map(|line| line.strip_prefix("#define HIP_VERSION_PATCH "))
        .expect("Invalid hip_version.h file structure: Patch version line not found")
        .parse::<u32>()
        .expect("Invalid hip_version.h file structure: Couldn't parse patch version");
    let release_patch = hip_header_patch_number_to_release_patch_number(system_patch);
    if release_patch.is_none() {
        println!("cargo::warning=Unknown release version for patch version {system_patch}. This patch does not correspond to an official release patch.");
    }

    Ok((system_major, system_minor, release_patch.unwrap_or(system_patch)))
}

/// The official patch number of a ROCm release is not the same of the patch number
/// in the header files. In the header files the patch number seems to be a monotonic
/// build number.
/// This function maps the header patch number to their official release number.
fn hip_header_patch_number_to_release_patch_number(number: u32) -> Option<u32> {
    match number {
        41134 => Some(4), // 6.2.4 (actually corresponds to most of 6.2.x versions for some reasons, we set to the last patch version)
        42131 => Some(0), // 6.3.0
        _ => None,
    }
}

/// Checks if the version inside `rocm_path` matches crate version
fn check_version(rocm_path: impl AsRef<Path>) -> std::io::Result<bool> {
    let (system_major, system_minor, system_patch) = get_system_hip_version(rocm_path)?;

    // Can be fairly sure that crate's versioning won't fail
    let crate_major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u8>().unwrap();
    let crate_minor = env!("CARGO_PKG_VERSION_MINOR").parse::<u8>().unwrap();
    // Need at least u32 here, because of the crates reserved versions for revisions,
    // and _unlikely_ possibility that ROCm's patch version will be higher than 16
    let crate_patch = env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap() / 1000;

    if crate_major == system_major {
        let mismatches = match (crate_minor == system_minor, crate_patch == system_patch) {
            // Perfect match, don't need a warning
            (true, true) => return Ok(true),
            (false, true) => "Minor",
            (true, false) => "Patch",
            (false, false) => "Both minor and patch",
        };
        println!("cargo::warning={mismatches} version mismatch between cubecl-hip-sys bindings and system HIP version. Want {}, but found {}",
            [crate_major, crate_minor, crate_patch as u8].map(|el| el.to_string()).join("."),
            [system_major, system_minor, system_patch as u8].map(|el| el.to_string()).join("."));
        Ok(true)
    } else {
        Ok(false)
    }
}

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

    let mut rocm_path_candidates = paths
        .iter()
        .filter(|path| {
            let hip_path = Path::new(path).join("include/hip");
            hip_path.exists()
        })
        .peekable();
    let have_candidates = rocm_path_candidates.peek().is_some();
    let rocm_path = rocm_path_candidates.find(|path| check_version(path).unwrap_or_default());

    if let Some(valid_rocm_path) = rocm_path {
        println!("cargo:rustc-link-lib=dylib=hiprtc");
        println!("cargo:rustc-link-lib=dylib=amdhip64");
        println!("cargo:rustc-link-search=native={}/lib", valid_rocm_path);
    } else if have_candidates {
        panic!(
            "None of the found ROCm installations match crate version {}",
            env!("CARGO_PKG_VERSION")
        );
    } else if paths.len() > 1 {
        panic!("HIP headers not found in any of the defined CUBECL_ROCM_PATH, ROCM_PATH or HIP_PATH directories.");
    }
}
