use chrono::{Datelike, NaiveDateTime};
use regex::Regex;
use serde_json::Value;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

const INPUT_FILE: &str = "missions.json";
const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const YEARS: [u32; 4] = [2021, 2020, 2019, 2018];
const CATID: u32 = 5;

lazy_static::lazy_static! {
    static ref OLD_WEBSITE_DIR: PathBuf = PathBuf::from("website.old");
    static ref OUTPUT_DIR: PathBuf = PathBuf::from("output");
    static ref CLEAN_REGEX: Regex = Regex::new("<[^<>]+>").unwrap(); // Remove HTML based stuff
    static ref IMAGE_REGEX: Regex = Regex::new("src=\"([^\"]+)\"").unwrap(); // Finds image source
    static ref NEW_LINE_AFTER_DOT_REGEX: Regex = Regex::new("([^0-9])(\\.\\s)").unwrap(); // One sentence per line
    static ref NEW_LINE_AT_BEGINING_REGEX: Regex = Regex::new("^(\n)+").unwrap(); // Find newlines at the begining
}

#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord)]
struct Article {
    pub title: String,
    pub date: String,
    pub text: String,
    pub images: Vec<PathBuf>,
}

struct YearArticles {
    pub year: u32,
    pub articles: Vec<Article>,
}

impl Article {
    fn to_markdown(&self, year: u32, index: usize) -> String {
        let mut output = String::new();
        let mut images_shortcodes = String::new();
        let formatted_article_index = Article::format_article_index(index);
        output.push_str("---\n");
        output.push_str(&format!("title: {}\n", self.title));
        output.push_str(&format!("date: {}\n", self.date));
        output.push_str(&format!("description: {}\n", self.title));
        if self.images.is_empty() {
            output.push_str("thumbnail: img/default.png\n")
        } else {
            output.push_str(&format!(
                "thumbnail: img/einsaetze/{}/{}.jpg\n",
                year, formatted_article_index
            ));
            output.push_str("resources:\n");
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
        }

        output.push_str("---\n\n");
        output.push_str(&self.text);
        output.push_str(&images_shortcodes);
        output
    }

    fn format_article_index(index: usize) -> String {
        format!("{:0>4}", index)
    }

    fn format_image_index(index: usize) -> String {
        format!("{:0>2}", index)
    }

    fn write(&self, article_dir: &Path, year: u32, article_index: usize) {
        let article_path = article_dir.join("index.md");
        if article_path.exists() {
            println!(
                "Article {}-{} already exists. Aborting!",
                year,
                Article::format_article_index(article_index)
            )
        } else {
            let article_markdown = self.to_markdown(year, article_index);
            fs::write(article_path, article_markdown).expect("Failed to write article");
        }
    }
}

impl YearArticles {
    fn write_articles(&self, output_dir: &Path) {
        let series_dir = output_dir.join("content").join(self.year.to_string());
        let thumbnail_dir = output_dir.join("thumbnail").join(self.year.to_string());

        if !series_dir.exists() {
            fs::create_dir_all(&series_dir).expect(&format!(
                "Failed to create the series directory {}",
                self.year
            ));

            fs::create_dir_all(&thumbnail_dir).expect(&format!(
                "Failed to create thumbnail directory {}",
                self.year
            ));

            for (article_index, article) in self.articles.iter().enumerate() {
                self.write_series_index(&series_dir);
                self.write_article(&series_dir, article, article_index);
                self.copy_thumbnail(&thumbnail_dir, article, article_index);
            }
        }
    }

    fn write_article(&self, article_year_dir: &Path, article: &Article, article_index: usize) {
        let article_dir = article_year_dir.join(Article::format_article_index(article_index));
        fs::create_dir(&article_dir).expect(&format!(
            "Failed to create article directory {}-{}",
            self.year,
            Article::format_article_index(article_index)
        ));
        article.write(&article_dir, self.year, article_index);
        let article_image_dir = article_dir.join("img");
        fs::create_dir(&article_image_dir).expect(&format!(
            "Failed to create image directory {}",
            article_image_dir.to_string_lossy()
        ));
        self.copy_images(&article_image_dir, article_index, &article.images);
    }

    fn write_series_index(&self, series_dir: &Path) {
        let series_index_path = series_dir.join("_index.md");
        let mut output = String::new();
        output.push_str("---\n");
        output.push_str(&format!("title: Einsätze {}\n", self.year));
        output.push_str("nested: false\n");
        output.push_str("---\n");
        fs::write(series_index_path, output)
            .expect(&format!("Failed to write series index {}", self.year));
    }

    fn copy_thumbnail(&self, thumbnail_dir: &Path, article: &Article, article_index: usize) {
        let source = article.images.first();
        if source.is_some() {
            let source = OLD_WEBSITE_DIR.join(source.unwrap());
            let destination = thumbnail_dir.join(&format!(
                "{}.jpg",
                Article::format_article_index(article_index)
            ));
            fs::copy(source, destination)
                .expect(&format!("Failed to copy thumbnail {}", article_index));
        }
    }

    fn copy_images(&self, article_image_dir: &Path, article_index: usize, images: &Vec<PathBuf>) {
        for (image_index, image_path) in images.iter().enumerate() {
            let image_name = format!(
                "{}-{}-{}.jpg",
                self.year,
                Article::format_article_index(article_index),
                Article::format_image_index(image_index)
            );
            let image_source = OLD_WEBSITE_DIR.join(image_path);
            let image_desination = article_image_dir.join(&image_name);
            fs::copy(&image_source, &image_desination).expect(&format!(
                "Failed to copy image {} to {}",
                image_source.to_string_lossy(),
                image_desination.to_string_lossy()
            ));
        }
    }
}

fn main() -> anyhow::Result<()> {
    let file = File::open(INPUT_FILE)?;
    let json: Value = serde_json::from_reader(file)?;
    let data = json["data"].as_array().unwrap();
    for year in YEARS {
        let year_articles = get_articles(data, year, CATID);
        year_articles.write_articles(&OUTPUT_DIR);
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
    let mut images: Vec<PathBuf> = Vec::default();
    let introtext = json["introtext"].as_str().expect("Inrtotext not found");
    let title = json["title"].as_str().expect("Title not found").to_string();
    let text = CLEAN_REGEX
        .replace_all(introtext, "")
        .to_string()
        .replace("\u{a0}", "")
        .replace("\r\n", "\n");

    let text = NEW_LINE_AFTER_DOT_REGEX
        .replace_all(&text, "${1}.\n")
        .to_string();
    let text = NEW_LINE_AT_BEGINING_REGEX.replace(&text, "").to_string();

    for capture in IMAGE_REGEX.captures_iter(introtext) {
        images.push(PathBuf::from(&capture[1]));
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
