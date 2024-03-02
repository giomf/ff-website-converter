use chrono::{Datelike, NaiveDateTime};
use regex::Regex;
use serde_json::Value;
use std::{
    fs::{self, File},
    path::Path,
};

const INPUT_FILE: &str = "input.json";
const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const YEARS: [u32; 3] = [2021, 2019, 2018];
const CATID: u32 = 5;
const OUTPUT_CONTENT_DIRECTORY: &str = "content";

#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord)]
struct Article {
    pub title: String,
    pub date: String,
    pub text: String,
    pub images: Vec<String>,
}

struct YearArticles {
    pub year: u32,
    pub articles: Vec<Article>,
}

impl Article {
    pub fn to_markdown(&self, year: u32, index: usize) -> String {
        let mut output = String::new();
        output.push_str("---\n");
        output.push_str(&format!("title: {}\n", self.title));
        output.push_str(&format!("date: {}\n", self.date));
        output.push_str(&format!("description: {}\n", self.title));
        output.push_str(&format!("thumbnail: img/einsaetze/{}/0000.jpg\n", year,));
        output.push_str("ressources:\n");

        let mut images_shortcodes = String::new();
        let formatted_article_index = Article::format_article_index(index);
        for image_index in 0..self.images.len() {
            let formatted_image_index = Article::format_image_index(image_index);
            output.push_str(&format!("- name: img-{}\n", formatted_image_index));
            output.push_str(&format!(
                "  src: img/{}-{}-{}.jpg\n",
                year, formatted_article_index, formatted_image_index
            ));
            images_shortcodes.push_str(&format!(
                "{{{{< image src=\"img-{}\" >}}}}  \n",
                formatted_image_index
            ));
        }
        output.push_str("---\n\n");
        output.push_str(&self.text);
        output.push_str(&images_shortcodes);
        output
    }

    pub fn format_article_index(index: usize) -> String {
        format!("{:0>4}", index)
    }

    pub fn format_image_index(index: usize) -> String {
        format!("{:0>2}", index)
    }

    pub fn write(&self, article_dir: &Path, year: u32, article_index: usize) {
        if article_dir.exists() {
            println!(
                "Article {}-{} already exists. Aborting!",
                year,
                Article::format_article_index(article_index)
            )
        } else {
            fs::create_dir(article_dir).expect("Failed to create article directory");
            let article_markdown = self.to_markdown(year, article_index);
            let article_path = article_dir.join("index.md");
            fs::write(article_path, article_markdown).expect("Failed to write article");
        }
    }
}

impl YearArticles {
    fn write(&self, content_path: &Path) -> anyhow::Result<()> {
        let artcile_year_dir = content_path.join(self.year.to_string());
        if !artcile_year_dir.exists() {
            fs::create_dir_all(&artcile_year_dir)?;
            for (article_index, article) in self.articles.iter().enumerate() {
                let article_dir =
                    artcile_year_dir.join(Article::format_article_index(article_index));
                article.write(&article_dir, self.year, article_index);
            }
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let file = File::open(INPUT_FILE)?;
    let json: Value = serde_json::from_reader(file)?;
    let data = json["data"].as_array().unwrap();
    let output_content_dir = Path::new(OUTPUT_CONTENT_DIRECTORY);
    for year in YEARS {
        let year_articles = get_articles(data, year, CATID);
        year_articles.write(output_content_dir)?;
    }
    Ok(())
}

fn get_articles(json: &Vec<Value>, year: u32, catid: u32) -> YearArticles {
    let mut articles: Vec<Article> = json
        .iter()
        .filter(|x| match (x["created"].as_str(), x["catid"].as_str()) {
            (Some(json_date), Some(json_catid)) => {
                let date = NaiveDateTime::parse_from_str(json_date, DATE_TIME_FORMAT).unwrap();
                date.year() as u32 == year && json_catid.parse::<u32>().unwrap() == catid
            }
            _ => false,
        })
        .map(|json_article| get_article(json_article))
        .collect();

    articles.sort_by_key(|x| x.date.clone());
    YearArticles { year, articles }
}

fn get_article(json: &Value) -> Article {
    let clean_re = Regex::new("<[^<>]+>").unwrap(); // Remove HTML based stuff
    let image_re = Regex::new("<img src=\"([^\"]+)\"").unwrap(); // Finds image source
    let new_line_after_dot_re = Regex::new("([^0-9])\\.(\\s)").unwrap(); // One sentence per line
    let new_line_re = Regex::new("^(\n)+").unwrap(); // Find newlines at the begining

    let mut images: Vec<String> = Vec::default();
    let introtext = json["introtext"].as_str().expect("Inrtotext not found");
    let title = json["title"].as_str().expect("Title not found").to_string();
    let text = clean_re
        .replace_all(introtext, "")
        .to_string()
        .replace("\u{a0}", "")
        .replace("\r\n", "\n");

    let text = new_line_after_dot_re.replace_all(&text, ".\n").to_string();
    let text = new_line_re.replace(&text, "").to_string();

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
