use clap::Parser;

use crate::ProfiledConfigArgs;

#[derive(Parser)]
#[command(version, about, long_about = "")]
pub struct ProfiledConfigParser {
    #[command(flatten)]
    pub profiled_config: ProfiledConfigArgs,
}
