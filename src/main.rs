use std::error::Error;

use regex::Regex;
use rss::Channel;
use scraper::{Html, Selector};

use cli::Cli;

mod cli;

fn main() -> Result<(), Box<dyn Error>> {
    run(&Cli::load());
    Ok(())
}

fn run(args: &Cli) {
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
    fn new(title: &str, uri: &str, size_str: &str) -> Self {
        let size = Self::convert_size(&size_str);
        println!("size: {}", size.unwrap());
        Magnet { title: title.to_string(), uri: uri.to_string(), size_str: size_str.to_string(), size: size.unwrap_or(0) }
    }

    fn convert_size(size_str: &str) -> Option<u32> {
        let re = Regex::new(r"([\d.]+)(.+)").unwrap();
        let cap = re.captures(size_str)?;
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


fn get_best_url(description: String, args: &Cli) -> Option<String> {
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

fn get_magnets(description: String, args: &Cli) -> Vec<Magnet> {
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
                result.push(Magnet::new(&cap[1], href, &cap[2]));
            }
        }
    }
    result
}

fn get_rss(args: &Cli) -> Result<Channel, Box<dyn Error>> {
    let resp = reqwest::blocking::get(&args.url)?.bytes()?;
    let channel = Channel::read_from(&(resp[..]))?;
    Ok(channel)
}
