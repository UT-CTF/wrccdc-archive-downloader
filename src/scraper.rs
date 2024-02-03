use std::ops::Deref;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use select::document::Document;
use select::predicate::Name;

#[derive(Debug)]
pub(crate) enum Entry {
    Directory(Box<str>),
    File(Box<str>),
}

pub(crate) async fn scrape(client: &Client, root: &str) -> Box<[Entry]> {
    let mut paths = Vec::new();
    paths.push(Box::from(root));

    let bar = ProgressBar::new(paths.len() as u64)
        .with_style(
            ProgressStyle::with_template(
                "{spinner} {msg} {wide_bar} {human_pos}/{human_len} {elapsed}",
            )
            .unwrap(),
        )
        .with_message("Scanning for Directories!");

    let mut entries = Vec::new();
    while let Some(path) = paths.pop() {
        enumerate_directory(&client, &path)
            .await
            .into_vec()
            .into_iter()
            .for_each(|entry| {
                if let Entry::Directory(path) = &entry {
                    bar.inc_length(1);
                    paths.push(path.clone());
                }
                entries.push(entry);
            });
        bar.inc(1);
    }
    bar.finish_with_message("Finished Enumerating Directories!");
    bar.finish_and_clear();
    entries.into_boxed_slice()
}

async fn enumerate_directory(client: &Client, path: &str) -> Box<[Entry]> {
    if path == "https://archive.wrccdc.org/images/2018/" {
        // FIXME https://archive.wrccdc.org/images/2018/wrccdc2018-bobscandy/
        // https://discord.com/channels/525435725123158026/525435725123158028/1203191151654469672
        return vec![].into_boxed_slice();
    }
    Document::from(
        client
            .get(path)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
            .deref(),
    )
    .find(Name("tbody"))
    .next()
    .unwrap()
    .find(Name("a"))
    .skip(1)
    .filter_map(|n| n.attr("href"))
    .map(|href| {
        if href.ends_with("/") {
            Entry::Directory(format!("{path}{href}").into_boxed_str())
        } else {
            Entry::File(format!("{path}{href}").into_boxed_str())
        }
    })
    .collect()
}
