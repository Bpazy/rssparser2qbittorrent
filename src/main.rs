use std::error::Error;

use clap::Parser;
use regex::Regex;
use rss::Channel;
use scraper::{Html, Selector};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    url: String,
    #[clap(short, long, value_parser)]
    best_regex: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    run(&Args::parse());
    Ok(())
}

fn run(args: &Args) {
    let channel = get_rss(&args).expect("Failed to get rss content");
    for item in channel.items {
        println!("Title: {}\n\tBest Url: {}", item.title.unwrap(), get_best_url(item.description.unwrap(), args).unwrap_or(String::from("None best found")));
    }
}

#[derive(Debug)]
struct Magnet {
    uri: String,
    title: String,
}

impl Magnet {
    fn new(title: String, uri: String) -> Self {
        Magnet { title, uri }
    }
}


fn get_best_url(description: String, args: &Args) -> Option<String> {
    let fragment = Html::parse_fragment(description.as_str());
    let selector = Selector::parse(r#"a[href]"#).unwrap();
    let mut v = Vec::new();
    for a in fragment.select(&selector) {
        match a.value().attr("href") {
            Some(href) => {
                if href.starts_with("magnet") {
                    v.push(Magnet::new(a.text().collect::<String>(), href.to_string()));
                }
            }
            _ => continue
        }
    }

    let re = Regex::new(args.best_regex.as_str()).unwrap();
    let mut best_magnet = None;
    for magnet in v {
        if re.find(magnet.title.as_str()).is_some() {
            best_magnet = Some(magnet)
        }
    }
    return match best_magnet {
        Some(magnet) => Some(magnet.uri),
        _ => None
    };
}

fn get_rss(args: &Args) -> Result<Channel, Box<dyn Error>> {
    let resp = reqwest::blocking::get(&args.url)?.bytes()?;
    let channel = Channel::read_from(&(resp[..]))?;
    Ok(channel)
}
