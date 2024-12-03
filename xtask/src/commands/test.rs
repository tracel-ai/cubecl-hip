use tracel_xtask::prelude::*;

#[macros::extend_command_args(TestCmdArgs, Target, TestSubCommand)]
pub struct CubeClHipTestCmdArgs {
    #[arg(long, short)]
    pub version: Option<String>,
}

pub(crate) fn handle_command(mut args: CubeClHipTestCmdArgs) -> anyhow::Result<()> {
    if let Some(version) = args.version.clone() {
        let feature_name = format!("rocm_{}", version.replace(".", ""));
        args.no_default_features = true;
        args.features = Some(vec![feature_name]);
    }
    base_commands::test::handle_command(args.try_into().unwrap())
}
