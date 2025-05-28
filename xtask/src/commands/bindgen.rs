use std::path::{Path, PathBuf};

use cubecl_hip_sys::hipconfig;
use tracel_xtask::{
    prelude::*,
    utils::workspace::{get_workspace_members, WorkspaceMember, WorkspaceMemberType},
};

#[derive(clap::Args)]
pub struct BindgenCmdArgs {
    /// Name of the crates for which we need to generate bindings. Pass "all" for all crates.
    #[arg(short, long, value_delimiter = ',', default_value = "cubecl-hip-sys")]
    crates: Vec<String>,
}

pub(crate) fn handle_command(args: BindgenCmdArgs) -> anyhow::Result<()> {
    run_bindgen(&args.crates)
}

fn run_bindgen(crates: &[String]) -> anyhow::Result<()> {
    let rocm_path = hipconfig::get_rocm_path()?;
    println!("rocm path: {rocm_path}");
    let include_path = hipconfig::get_hip_include_path()?;
    println!("hip include path: {include_path}");
    let hip_patch = hipconfig::get_hip_patch_version()?;
    println!("hip patch: {hip_patch}");
    let members = get_workspace_members(WorkspaceMemberType::Crate);
    for member in members {
        if member.name == "all" || crates.contains(&member.name) {
            group_info!("Generate bindings: {}", member.name);
            let header_path = get_wrapper_file_path(&member)?;
            let bindings_path = get_bindings_file_path(&member, &hip_patch)?;
            println!("bindings path: {bindings_path}");
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

fn get_bindings_file_path(member: &WorkspaceMember, patch: &str) -> anyhow::Result<String> {
    let out_path = get_output_path(member)?;
    let path = out_path.join(format!("bindings_{patch}.rs"));
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
