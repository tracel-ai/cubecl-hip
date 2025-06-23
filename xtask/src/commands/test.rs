use std::path::PathBuf;

use tracel_xtask::prelude::*;

#[macros::extend_command_args(TestCmdArgs, Target, TestSubCommand)]
pub struct CubeClHipTestCmdArgs {
    /// Override HIP_PATH to the specified path.
    #[arg(long = "path", short = 'p')]
    pub hip_path: Option<PathBuf>,
}

pub(crate) fn handle_command(
    args: CubeClHipTestCmdArgs,
    env: Environment,
    ctx: Context,
) -> anyhow::Result<()> {
    if let Some(path) = args.hip_path.clone() {
        std::env::set_var("HIP_PATH", path);
    }
    base_commands::test::handle_command(args.try_into().unwrap(), env, ctx)
}
