use chrono::{Datelike, NaiveDateTime};
use regex::Regex;
use serde_json::Value;
use std::fs::File;

const INPUT_FILE: &str = "input.json";
const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Default, Clone)]
struct Article {
    pub title: String,
    pub date: String,
    pub text: String,
    pub images: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let file = File::open(INPUT_FILE)?;
    let json: Value = serde_json::from_reader(file)?;
    let data = json["data"].as_array().unwrap();
    let data_2021 = get_articles(data, 2021, 5);
    Ok(())
}

fn get_articles(json: &Vec<Value>, year: i32, catid: i32) -> Vec<Article> {
    json.iter()
        .filter(|x| match (x["created"].as_str(), x["catid"].as_str()) {
            (Some(json_date), Some(json_catid)) => {
                let date = NaiveDateTime::parse_from_str(json_date, DATE_TIME_FORMAT).unwrap();
                date.year() == year && json_catid.parse::<i32>().unwrap() == catid
            }
            _ => false,
        })
        .map(|json_article| get_article(json_article))
        .collect()
}

fn get_article(json: &Value) -> Article {
    let clean_re = Regex::new("<[^<>]+>").unwrap();
    let image_re = Regex::new("<img src=\"([^\"]+)\"").unwrap();

    let mut images: Vec<String> = Vec::default();
    let introtext = json["introtext"].as_str().expect("Inrtotext not found");
    let title = json["title"].as_str().expect("Title not found").to_string();
    let text = clean_re
        .replace_all(introtext, "")
        .to_string()
        .replace("\u{a0}", "")
        .replace("\r\n", "\n");

    for capture in image_re.captures_iter(introtext) {
        images.push(capture[1].to_string());
    }

    let date = json["created"]
        .as_str()
        .expect("Created not found")
        .to_string();

    Article {
        title,
        date,
        text,
        images,
    }
}
