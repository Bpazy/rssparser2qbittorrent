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
    size_str: String,
    size: u32,
}

impl Magnet {
    fn new(title: String, uri: String, size_str: String) -> Self {
        let size = Self::convert_size(&size_str);
        println!("size: {}", size.unwrap());
        Magnet { title, uri, size_str, size: size.unwrap_or(0) }
    }

    fn convert_size(size_str: &String) -> Option<u32> {
        let re = Regex::new(r"([\d.]+)(.+)").unwrap();
        let cap = re.captures(size_str.as_str())?;
        if cap.len() <= 2 {
            return None;
        }
        let result = match cap[2].to_string().as_str() {
            "GB" => Some((cap[1].parse().unwrap_or(0.0) * (1024 * 1024) as f64) as u32),
            _ => None
        };
        return result;
    }
}


fn get_best_url(description: String, args: &Args) -> Option<String> {
    let magnets = get_magnets(description, args);
    Some(get_best_magnet(magnets)?.uri)
}

fn get_best_magnet(magnets: Vec<Magnet>) -> Option<Magnet> {
    let mut result: Option<Magnet> = None;
    for magnet in magnets {
        if magnet.size < 3 * 1024 * 1024 {
            continue;
        }
        result = match result {
            Some(best_magnet) => {
                if best_magnet.size < magnet.size {
                    return Some(magnet);
                }
                Some(best_magnet)
            }
            _ => Some(magnet)
        }
    }
    result
}

fn get_magnets(description: String, args: &Args) -> Vec<Magnet> {
    let fragment = Html::parse_fragment(description.as_str());
    let selector = Selector::parse(r#"a[href]"#).unwrap();
    let re = Regex::new(args.best_regex.as_str()).unwrap();
    let mut result = Vec::new();
    for a in fragment.select(&selector) {
        let href = a.value().attr("href");
        if href.is_none() {
            continue;
        }
        let href = href.unwrap();
        if href.starts_with("magnet") {
            let str = a.text().collect::<String>();
            if let Some(cap) = re.captures(str.as_str()) {
                result.push(Magnet::new(cap[1].to_string(), href.to_string(), cap[2].to_string()));
            }
        }
    }
    result
}

fn get_rss(args: &Args) -> Result<Channel, Box<dyn Error>> {
    let resp = reqwest::blocking::get(&args.url)?.bytes()?;
    let channel = Channel::read_from(&(resp[..]))?;
    Ok(channel)
}
