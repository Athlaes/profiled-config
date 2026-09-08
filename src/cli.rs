use clap::Parser;

use crate::api::ProfiledConfigArgs;

#[derive(Parser)]
#[command(version, about, long_about = "")]
pub struct ProfiledConfigParser {
    #[command(flatten)]
    pub profiled_config: ProfiledConfigArgs,
}
