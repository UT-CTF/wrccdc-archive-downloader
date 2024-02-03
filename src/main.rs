use std::fs::{create_dir_all, remove_dir_all};
use std::ops::Deref;
use std::path::Path;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;

use self::scraper::scrape;
use crate::downloader::download;
use crate::scraper::Entry;

mod downloader;
mod scraper;

#[tokio::main]
async fn main() {
    const ROOT: &str = "https://archive.wrccdc.org/images/";

    let client = Client::builder().http3_prior_knowledge().build().unwrap();

    let entries = scrape(&client, ROOT).await;
    let urls: Vec<Box<str>> = entries
        .into_vec()
        .into_iter()
        .filter_map(|entry| match entry {
            Entry::Directory(url) => {
                create_dir_all(format!("images/{}", url.split_at(ROOT.len()).1)).unwrap();
                None
            }
            Entry::File(url) => Some(url),
        })
        .collect();

    let progress = MultiProgress::new();
    let overall_progress = progress.add(
        ProgressBar::new(urls.len() as u64).with_style(
            ProgressStyle::default_bar()
                .template("{spinner} Downloading Files ({pos}/{len}): {wide_bar}")
                .unwrap(),
        ),
    );

    create_dir_all("temp").unwrap();

    for url in urls.into_iter() {
        let path = Box::from(Path::new(
            format!("images/{}", url.split_at(ROOT.len()).1).as_str(),
        ));
        download(&client, url.deref(), path, &progress).await;
        overall_progress.inc(1);
    }

    remove_dir_all("temp").unwrap();
}
