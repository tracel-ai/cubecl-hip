use std::fmt;
use std::path::Path;

pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u32,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Reads the header inside the rocm folder that contains the ROCm global version
pub fn get_rocm_system_version(rocm_path: impl AsRef<Path>) -> std::io::Result<Version> {
    let version_path = rocm_path.as_ref().join("include/rocm-core/rocm_version.h");
    let version_file = std::fs::read_to_string(version_path)?;
    let version_lines = version_file.lines().collect::<Vec<_>>();

    let major = version_lines
        .iter()
        .find_map(|line| line.strip_prefix("#define ROCM_VERSION_MAJOR "))
        .expect("Invalid rocm_version.h file structure: Major version line not found.")
        .trim()
        .parse::<u8>()
        .expect("Invalid rocm_version.h file structure: Couldn't parse major version.");
    let minor = version_lines
        .iter()
        .find_map(|line| line.strip_prefix("#define ROCM_VERSION_MINOR "))
        .expect("Invalid rocm_version.h file structure: Minor version line not found.")
        .trim()
        .parse::<u8>()
        .expect("Invalid rocm_version.h file structure: Couldn't parse minor version.");
    let patch = version_lines
        .iter()
        .find_map(|line| line.strip_prefix("#define ROCM_VERSION_PATCH "))
        .expect("Invalid rocm_version.h file structure: Patch version line not found.")
        .trim()
        .parse::<u32>()
        .expect("Invalid rocm_version.h file structure: Couldn't parse patch version.");

    Ok(Version {
        major,
        minor,
        patch,
    })
}

/// Reads the HIP header inside the rocm folder that contains the HIP specific version
pub fn get_hip_system_version(rocm_path: impl AsRef<Path>) -> std::io::Result<Version> {
    let version_path = rocm_path.as_ref().join("include/hip/hip_version.h");
    let version_file = std::fs::read_to_string(version_path)?;
    let version_lines = version_file.lines().collect::<Vec<_>>();

    let major = version_lines
        .iter()
        .find_map(|line| line.strip_prefix("#define HIP_VERSION_MAJOR "))
        .expect("Invalid hip_version.h file structure: Major version line not found.")
        .trim()
        .parse::<u8>()
        .expect("Invalid hip_version.h file structure: Couldn't parse major version.");
    let minor = version_lines
        .iter()
        .find_map(|line| line.strip_prefix("#define HIP_VERSION_MINOR "))
        .expect("Invalid hip_version.h file structure: Minor version line not found.")
        .trim()
        .parse::<u8>()
        .expect("Invalid hip_version.h file structure: Couldn't parse minor version.");
    let patch = version_lines
        .iter()
        .find_map(|line| line.strip_prefix("#define HIP_VERSION_PATCH "))
        .expect("Invalid hip_version.h file structure: Patch version line not found.")
        .trim()
        .parse::<u32>()
        .expect("Invalid hip_version.h file structure: Couldn't parse patch version.");

    Ok(Version {
        major,
        minor,
        patch,
    })
}
