use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'n', long = "no-open", default_value_t = false)]
    pub no_open: bool,
}

pub fn parse() -> Args {
    let args = Args::parse();
    return args;
}
