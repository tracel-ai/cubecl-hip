use std::env;

const ROCM_FEATURE_PREFIX: &str = "CARGO_FEATURE_ROCM__";
const ROCM_HIP_FEATURE_PREFIX: &str = "CARGO_FEATURE_HIP_";

include!("../build-script/src/lib.rs");

/// Make sure that at least one and only one hip feature is set
fn ensure_single_rocm_hip_feature_set() {
    let mut enabled_features = Vec::new();

    for (key, value) in env::vars() {
        if key.starts_with(ROCM_HIP_FEATURE_PREFIX) && value == "1" {
            enabled_features.push(format!(
                "rocm__{}",
                key.strip_prefix(ROCM_HIP_FEATURE_PREFIX).unwrap()
            ));
        }
    }

    match enabled_features.len() {
        1 => {}
        0 => panic!("No ROCm HIP feature is enabled. One ROCm HIP feature must be set."),
        _ => panic!(
            "Multiple ROCm HIP features are enabled: {:?}. Only one can be set.",
            enabled_features
        ),
    }
}

/// Checks if the version inside `rocm_path` matches crate version
fn check_rocm_version(rocm_path: impl AsRef<Path>) -> std::io::Result<bool> {
    let rocm_system_version = get_rocm_system_version(rocm_path)?;
    let rocm_feature_version = get_rocm_feature_version();

    if rocm_system_version.major == rocm_feature_version.major {
        let mismatches = match (
            rocm_system_version.minor == rocm_feature_version.minor,
            rocm_system_version.patch == rocm_feature_version.patch,
        ) {
            // Perfect match, don't need a warning
            (true, true) => return Ok(true),
            (true, false) => "Patch",
            (false, _) => "Minor",
        };
        println!("cargo::warning=ROCm {mismatches} version mismatch between cubecl-hip-sys expected version ({rocm_feature_version}) and found ROCm version on the system ({rocm_system_version}). Build process might fail due to incompatible library bindings.");
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Return the ROCm version corresponding to the enabled rocm__<version> feature
fn get_rocm_feature_version() -> Version {
    for (key, value) in env::vars() {
        if key.starts_with(ROCM_FEATURE_PREFIX) && value == "1" {
            if let Some(version) = key.strip_prefix(ROCM_FEATURE_PREFIX) {
                let parts: Vec<&str> = version.split('_').collect();
                if parts.len() == 3 {
                    if let (Ok(major), Ok(minor), Ok(patch)) = (
                        parts[0].parse::<u8>(),
                        parts[1].parse::<u8>(),
                        parts[2].parse::<u32>(),
                    ) {
                        return Version {
                            major,
                            minor,
                            patch,
                        };
                    }
                }
            }
        }
    }

    panic!("No valid ROCm feature version found. One 'rocm__<version>' feature must be set. For instance for ROCm 6.2.2 the feature is rocm__6_2_2.")
}

/// Return the ROCm HIP patch version corresponding to the enabled hip_<patch_version> feature
fn get_hip_feature_patch_version() -> u32 {
    for (key, value) in env::vars() {
        if key.starts_with(ROCM_HIP_FEATURE_PREFIX) && value == "1" {
            if let Some(patch) = key.strip_prefix(ROCM_HIP_FEATURE_PREFIX) {
                if let Ok(patch) = patch.parse::<u32>() {
                    return patch;
                }
            }
        }
    }

    panic!("No valid ROCm HIP feature found. One 'hip_<patch>' feature must be set.")
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=CUBECL_ROCM_PATH");
    println!("cargo::rerun-if-env-changed=ROCM_PATH");
    println!("cargo::rerun-if-env-changed=HIP_PATH");

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
    let rocm_path = rocm_path_candidates.find(|path| check_rocm_version(path).unwrap_or_default());

    if let Some(valid_rocm_path) = rocm_path {
        ensure_single_rocm_hip_feature_set();
        // verify HIP compatibility
        let Version {
            patch: hip_system_patch_version,
            ..
        } = get_hip_system_version(valid_rocm_path).unwrap();
        let hip_feature_patch_version = get_hip_feature_patch_version();
        if hip_system_patch_version != hip_feature_patch_version {
            panic!("Incompatible HIP bindings found. Expected to find HIP patch version {hip_feature_patch_version}, but found HIP patch version {hip_system_patch_version}.");
        }

        println!("cargo::rustc-link-lib=dylib=hiprtc");
        println!("cargo::rustc-link-lib=dylib=amdhip64");
        println!("cargo::rustc-link-search=native={}/lib", valid_rocm_path);
    } else if have_candidates {
        let rocm_feature_version = get_rocm_feature_version();
        panic!("None of the found ROCm installations match version {rocm_feature_version}.");
    } else if paths.len() > 1 {
        panic!("HIP headers not found in any of the directories set in CUBECL_ROCM_PATH, ROCM_PATH or HIP_PATH environment variable.");
    }
}
