use std::fmt;
use std::path::Path;
use std::process::Command;

use regex::Regex;

pub const HIPCONFIG: &str = "hipconfig";
const ROCM_HIP_FEATURE_PREFIX: &str = "CARGO_FEATURE_HIP_";

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

/// Retrieve the HIP_PATH from the `hipconfig -p` utility
pub fn get_hip_path_from_hipconfig() -> std::io::Result<String> {
    let output = Command::new(HIPCONFIG).arg("-p").output()?;
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(path)
}

/// Retrieve the HIP patch number from the `hipconfig --version` output
pub fn get_hip_patch_version_from_hipconfig() -> std::io::Result<String> {
    let output = Command::new(HIPCONFIG).arg("--version").output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_hip_build_number(&stdout)
}

/// extract the HIP patch number from hipconfig version output
pub fn parse_hip_build_number(version_output: &str) -> std::io::Result<String> {
    let re = Regex::new(r"\d+\.\d+\.(\d+)-").expect("regex should compile");
    if let Some(caps) = re.captures(version_output) {
        if let Some(m) = caps.get(1) {
            return Ok(m.as_str().to_string());
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "should extract HIP build number from version output",
    ))
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

/// Return the ROCm HIP patch version corresponding to the enabled hip_<patch_version> feature
pub fn get_hip_feature_patch_version() -> String {
    for (key, value) in std::env::vars() {
        if let Some(patch) = parse_hip_feature_patch(&key, &value) {
            return patch;
        }
    }
    panic!("No valid ROCm HIP feature found. One `hip_<patch>` feature must be set.");
}

/// Extract the patch number of the hip feature as a string
fn parse_hip_feature_patch(key: &str, value: &str) -> Option<String> {
    // value 1 means the feature is enabled
    // the actual feature name is in the key part
    // the feature name contains the HIP patch version
    if key.starts_with(ROCM_HIP_FEATURE_PREFIX) && value == "1" {
        key.strip_prefix(ROCM_HIP_FEATURE_PREFIX)
            .map(|patch| patch.to_string())
    } else {
        None
    }
}

/// Return the library directory using hipconfig
pub fn get_rocm_library_directory_name() -> std::io::Result<String> {
    let output = Command::new(HIPCONFIG).arg("-R").output()?;
    let rocm_path = String::from_utf8_lossy(&output.stdout);
    let output = Command::new(HIPCONFIG).arg("-l").output()?;
    let clang_path = String::from_utf8_lossy(&output.stdout);
    parse_rocm_library_directory_name(&rocm_path, &clang_path)
}

/// Parse out the first subdirectory under the ROCm root path
pub fn parse_rocm_library_directory_name(
    rocm_path: &str,
    clang_path: &str,
) -> std::io::Result<String> {
    let rocm = rocm_path.trim().trim_end_matches('/');
    let clang = clang_path.trim();

    // Build a regex like "^/opt/rocm/([^/]+)"
    let pattern = format!(r"^{}\/([^/]+)", regex::escape(rocm));
    let re = Regex::new(&pattern).expect("regex should compile");

    if let Some(caps) = re.captures(clang) {
        // Group 1 is the first directory after the rocM root
        return Ok(caps[1].to_string());
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "failed to extract library directory from clang path",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::io::ErrorKind;

    #[rstest]
    #[case::standard("6.4.43482-0f2d60242", Some("43482"))]
    #[case::with_rc_suffix("10.20.54321-rc1", Some("54321"))]
    #[case::leading_zeros("6.4.00099-test", Some("00099"))]
    #[case::missing_hyphen("6.4.43482", None)]
    #[case::completely_invalid("no numbers", None)]
    fn test_parse_hip_build_number(#[case] input: &str, #[case] expected: Option<&str>) {
        let result = parse_hip_build_number(input);

        match expected {
            Some(expected) => {
                let got = result.expect("should parse build number from version output");
                assert_eq!(got, expected, "parsed build number should match expected");
            }
            None => {
                let err = result.expect_err("should fail to parse invalid version output");
                assert_eq!(
                    err.kind(),
                    ErrorKind::InvalidData,
                    "error kind should be InvalidData"
                );
            }
        }
    }

    #[rstest]
    #[case::hip_41134("CARGO_FEATURE_HIP_41134", "1", Some("41134"))]
    #[case::hip_42131("CARGO_FEATURE_HIP_42131", "1", Some("42131"))]
    #[case::leading_zero("CARGO_FEATURE_HIP_00456", "1", Some("00456"))]
    #[case::disabled_feature("CARGO_FEATURE_HIP_42131", "0", None)]
    #[case::wrong_prefix("OTHER_CARGO_FEATURE_HIP_12345", "1", None)]
    #[case::wrong_feature_name("OTHER_CARGO_FEATURE_HIP__12345", "1", None)]
    #[case::empty_value("CARGO_FEATURE_HIP_12345", "", None)]
    fn test_parse_hip_feature_patch(
        #[case] key: &str,
        #[case] value: &str,
        #[case] expected: Option<&str>,
    ) {
        let result = parse_hip_feature_patch(key, value);
        assert_eq!(
            result.as_deref(),
            expected,
            "for key=`{key}`, value=`{value}`, expected={expected:?}, got={result:?}",
        );
    }

    #[rstest]
    #[case::standard("/opt/rocm\n", "/opt/rocm/lib/llvm/bin\n", Some("lib"))]
    #[case::lib64("/usr/local/rocm\n", "/usr/local/rocm/lib64/x86_64\n", Some("lib64"))]
    #[case::trailing_slash("/opt/rocm/\n", "/opt/rocm/lib/foo\n", Some("lib"))]
    #[case::no_match("/opt/rocm\n", "/other/path/lib\n", None)]
    #[case::no_component("/opt/rocm\n", "/opt/rocm/\n", None)]
    fn test_parse_rocm_library_directory_name(
        #[case] rocm: &str,
        #[case] clang: &str,
        #[case] expected: Option<&str>,
    ) {
        let result = parse_rocm_library_directory_name(rocm, clang);

        match expected {
            Some(expected) => {
                let got =
                    result.expect("should extract the library subdirectory from the clang path");
                assert_eq!(
                    got, expected,
                    "parsed directory should match the expected name"
                );
            }
            None => {
                let err =
                    result.expect_err("should fail when no valid library directory is present");
                assert_eq!(
                    err.kind(),
                    ErrorKind::InvalidData,
                    "error kind should be InvalidData"
                );
            }
        }
    }
}
