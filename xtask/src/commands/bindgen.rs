use std::path::{Path, PathBuf};

use tracel_xtask::{
    prelude::*,
    utils::workspace::{get_workspace_members, WorkspaceMember, WorkspaceMemberType},
};

#[derive(clap::Args)]
pub struct BindgenCmdArgs {
    /// Name of the crates for which we need to generate bindings. Pass "all" for all crates.
    #[arg(short, long, value_delimiter = ',')]
    crates: Vec<String>,
    /// Base installation path of the ROCm libraries installation.
    #[arg(short = 'p', long, default_value = "/opt")]
    installation_path: String,
    /// Version of the ROCm libraries to generate bindings for.
    #[arg(short, long)]
    version: String,
}

pub(crate) fn handle_command(args: BindgenCmdArgs) -> anyhow::Result<()> {
    run_bindgen(&args.crates, &args.installation_path, &args.version)
}

fn run_bindgen(crates: &[String], installation_path: &str, version: &str) -> anyhow::Result<()> {
    let include_path = get_include_path(installation_path, version)?;
    let members = get_workspace_members(WorkspaceMemberType::Crate);
    for member in members {
        if member.name == "all" || crates.contains(&member.name) {
            group_info!("Generate bindings: {}", member.name);
            let header_path = get_wrapper_file_path(&member)?;
            let bindings_path = get_bindings_file_path(&member, version)?;
            // Generate bindings using bindgen
            let bindings = bindgen::Builder::default()
                .header(header_path)
                .clang_arg("-D__HIP_PLATFORM_AMD__")
                .clang_arg(format!("-I{}", include_path))
                .layout_tests(false)
                .generate()
                .expect("Should generate HIP RTC bindings");
            bindings
                .write_to_file(bindings_path)
                .expect("Should write bindings file");
            endgroup!();
        } else {
            group_info!("Skip '{}' because it has been excluded!", &member.name);
        }
    }
    Ok(())
}

fn get_rocm_path(installation_path: &str, version: &str) -> anyhow::Result<PathBuf> {
    let path = Path::new(installation_path).join(format!("rocm-{}", version));
    if path.exists() {
        Ok(path)
    } else {
        Err(anyhow::anyhow!(
            "Cannot find ROCm base installation path on your system: {}",
            path.display()
        ))
    }
}

fn get_include_path(installation_path: &str, version: &str) -> anyhow::Result<String> {
    let path = get_rocm_path(installation_path, version)?.join("include");
    if path.exists() {
        Ok(path.to_string_lossy().into_owned())
    } else {
        Err(anyhow::anyhow!(
            "Cannot find include path on your system: {}",
            path.display()
        ))
    }
}

fn get_output_path(member: &WorkspaceMember) -> anyhow::Result<PathBuf> {
    let path = Path::new(&member.path).join("src").join("bindings");
    if path.exists() {
        Ok(path)
    } else {
        Err(anyhow::anyhow!(
            "Cannot find output path: {}",
            path.display()
        ))
    }
}

fn get_bindings_file_path(member: &WorkspaceMember, version: &str) -> anyhow::Result<String> {
    let out_path = get_output_path(member)?;
    let path = out_path.join(format!("bindings_{}.rs", version.replace('.', "")));
    Ok(path.to_string_lossy().into_owned())
}

fn get_input_path(member: &WorkspaceMember) -> anyhow::Result<PathBuf> {
    let path = Path::new(&member.path).to_path_buf();
    if path.exists() {
        Ok(path)
    } else {
        Err(anyhow::anyhow!(
            "Cannot find input path: {}",
            path.display()
        ))
    }
}

fn get_wrapper_file_path(member: &WorkspaceMember) -> anyhow::Result<String> {
    let out_path = get_input_path(member)?;
    let path = out_path.join("wrapper.h");
    Ok(path.to_string_lossy().into_owned())
}
