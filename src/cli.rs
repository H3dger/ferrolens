use std::path::PathBuf;

use clap::Parser;

use crate::theme::ThemeName;

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(name = "ferrolens")]
pub struct Cli {
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    #[arg(long, value_enum, default_value_t = ThemeName::Default)]
    pub theme: ThemeName,
}

pub fn parse_from<I, T>(itr: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    Cli::try_parse_from(itr)
}
