use tracel_xtask::prelude::*;

pub(crate) fn handle_command(mut args: TestCmdArgs) -> anyhow::Result<()> {
    args.no_default_features = true;
    base_commands::test::handle_command(args)
}
