use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'n', long = "no-open", default_value_t = true)]
    pub no_open: bool,
}
