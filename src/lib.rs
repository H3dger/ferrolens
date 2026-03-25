pub mod app;
pub mod cli;
pub mod data;
pub mod error;
pub mod export;
pub mod filter;
pub mod input;
pub mod theme;
pub mod tui;
pub mod ui;

use app::App;
use error::Result;

pub fn run_with_args<I, T>(itr: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    match cli::parse_from(itr) {
        Ok(cli) => {
            let dataset = data::loader::load_dataset(&cli.input)?;
            let app = App::with_theme(dataset, cli.theme);
            tui::run(app, cli.input.display().to_string())
        }
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            let _ = error.print();
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}
