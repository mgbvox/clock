use crate::VERSION;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "clock",
    version = VERSION,
    author = "Matthew Billman",
    about = "Clock in/out for work",
    disable_version_flag = true
)]
pub struct Cli {
    /// print version
    #[command(subcommand)]
    pub command: Cmds,
}

#[derive(Subcommand, Debug)]
pub enum Cmds {
    Version,
    In(InCmd),
    Out(OutCmd),
    /// Watch the active session, refreshing every -n seconds
    Watch {
        #[arg(short, default_value = "1")]
        n: u64,
    },
    LS,
}

#[derive(Parser, Debug)]
pub struct InCmd {
    /// Clock in with a job name
    #[arg(short, long, value_name = "JOBNAME")]
    pub job: String,
}

#[derive(Parser, Debug)]
pub struct OutCmd {
    /// Message to add to the clock-out log
    #[arg(short = 'm', value_name = "MESSAGE")]
    pub message: String,
}

#[derive(Parser, Debug)]
pub struct IOCmd {
    /// List all clock-in and clock-out records
    #[arg(short = 'l', long = "ls")]
    pub list: bool,

    /// Watch refresh
    #[arg(
        short = 'n',
        long = "refresh",
        value_name = "REFRESH",
        default_value = "1"
    )]
    pub n: u64,

    /// Export records
    #[arg(short = 'x', long = "export", value_name = "EXPORT")]
    pub export_filter: Option<String>,
}
