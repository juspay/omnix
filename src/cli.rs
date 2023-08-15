use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Do not automatically open the application in the local browser
    #[arg(short = 'n', long = "no-open", default_value_t = true)]
    pub no_open: bool,
}
