use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(short, long)]
    pub url: String,
    #[clap(short, long)]
    pub best_regex: String,
}

impl Cli {
    pub fn load() -> Cli {
        Cli::parse()
    }
}
