use std::collections::HashMap;
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get("https://www.baidu.com")?
        .json::<HashMap<String, String>>()?;
    println!("{:#?}", resp);

    Ok(())
}
