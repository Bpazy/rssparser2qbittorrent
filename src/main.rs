use clap::Parser;
use rss::Channel;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    url: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("args.name: {}", args.url);

    let resp = reqwest::blocking::get(args.url)?.bytes()?;
    let channel = Channel::read_from(&resp[..])?;
    for item in channel.items {
        println!("{}", item.title.unwrap());
    }

    Ok(())
}
