use std::process::Command;

use regex::Regex;

pub const HIPCONFIG: &str = "hipconfig";

/// Retrieve the ROCM_PATH with `hipconfig -R` command.
pub fn get_rocm_path() -> String {
    exec_hipconfig(&["-R"]).unwrap()
}

/// Retrieve the HIP_PATH with `hipconfig -p` command.
pub fn get_hip_path() -> String {
    exec_hipconfig(&["-p"]).unwrap()
}

/// Retrieve the HIP patch number from the `hipconfig --version` output
pub fn get_hip_patch_version() -> String {
    let hip_version = exec_hipconfig(&["--version"]).unwrap();
    parse_hip_patch_number(&hip_version)
}

/// Return the HIP path suitable for LD_LIBRARY_PATH.
pub fn get_hip_ld_library_path() -> String {
    let rocm_path = get_rocm_path();
    let lib_dir = get_hip_library_directory_name(&rocm_path);
    format!("{rocm_path}/{lib_dir}")
}

/// Execute hipconfig
fn exec_hipconfig(args: &[&str]) -> std::io::Result<String> {
    match Command::new(HIPCONFIG).args(args).output() {
        Ok(output) => {
            if output.stderr.is_empty() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                panic!(
                    "Error executing {HIPCONFIG}. The process returned:\n{}",
                    String::from_utf8_lossy(&output.stderr).trim()
                );
            }
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            panic!("Could not find '{HIPCONFIG}' in your PATH. You should install ROCm HIP or ensure '{HIPCONFIG}' is available. Fro more information please visit https://rocm.docs.amd.com/projects/install-on-linux/en/latest/.");
        }
        Err(e) => panic!(
            "Failed to run '{HIPCONFIG}' with args '{args:?}', reason: {}",
            e
        ),
    }
}

/// extract the HIP patch number from hipconfig version output
fn parse_hip_patch_number(version: &str) -> String {
    let re = Regex::new(r"\d+\.\d+\.(\d+)-").expect("regex should compile");
    if let Some(caps) = re.captures(version) {
        if let Some(m) = caps.get(1) {
            return m.as_str().to_string();
        }
    }
    // cannot parse for the patch number
    panic!("Error retrieving HIP patch number from value '{version}'")
}

/// Return the library directory using hipconfig
fn get_hip_library_directory_name(rocm_path: &str) -> String {
    let clang_path = exec_hipconfig(&["-l"]).unwrap();
    parse_hip_library_directory_name(rocm_path, &clang_path)
}

/// Parse out the first subdirectory under the ROCm path
fn parse_hip_library_directory_name(rocm_path: &str, clang_path: &str) -> String {
    let rocm = rocm_path.trim().trim_end_matches('/');
    let clang = clang_path.trim();
    // Build a regex like "^/opt/rocm/([^/]+)"
    let pattern = format!(r"^{}\/([^/]+)", regex::escape(rocm));
    let re = Regex::new(&pattern).expect("regex should compile");

    if let Some(caps) = re.captures(clang) {
        return caps[1].to_string();
    }
    panic!("Cannot retrieve the name of the HIP library directoy.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case::standard("6.4.43482-0f2d60242", Some("43482"))]
    #[case::with_rc_suffix("10.20.54321-rc1", Some("54321"))]
    #[case::leading_zeros("6.4.00099-test", Some("00099"))]
    #[case::missing_hyphen("6.4.43482", None)]
    #[case::completely_invalid("no numbers", None)]
    fn test_parse_hip_patch_number(#[case] input: &str, #[case] expected: Option<&str>) {
        let result = std::panic::catch_unwind(|| parse_hip_patch_number(input));
        match expected {
            Some(expected_str) => {
                let output = result.expect("should not panic for valid version");
                assert_eq!(
                    output, expected_str,
                    "parsed patch number should match expected"
                );
            }
            None => {
                assert!(result.is_err(), "should panic for invalid version output");
            }
        }
    }

    #[rstest]
    #[case::standard("/opt/rocm\n", "/opt/rocm/lib/llvm/bin\n", Some("lib"))]
    #[case::lib64("/usr/local/rocm\n", "/usr/local/rocm/lib64/x86_64\n", Some("lib64"))]
    #[case::trailing_slash("/opt/rocm/\n", "/opt/rocm/lib/foo\n", Some("lib"))]
    #[case::no_match("/opt/rocm\n", "/other/path/lib\n", None)]
    #[case::no_component("/opt/rocm\n", "/opt/rocm/\n", None)]
    fn test_parse_hip_library_directory_name(
        #[case] rocm: &str,
        #[case] clang: &str,
        #[case] expected: Option<&str>,
    ) {
        let result = std::panic::catch_unwind(|| parse_hip_library_directory_name(rocm, clang));
        match expected {
            Some(expected_dir) => {
                // For valid inputs, it must not panic and return the directory name
                let got = result.expect("should not panic for valid paths");
                assert_eq!(
                    got, expected_dir,
                    "parsed directory should match the expected name"
                );
            }
            None => {
                // For invalid inputs, it must panic
                assert!(
                    result.is_err(),
                    "should panic when no valid library directory is present"
                );
            }
        }
    }
}
