use std::ops::Deref;
use std::path::Path;

use futures_util::stream::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use tokio::fs::{remove_file, File};
use tokio::io::{AsyncWriteExt, BufWriter};

pub(crate) async fn download(
    client: &Client,
    url: &str,
    path: Box<Path>,
    progress: &MultiProgress,
) {
    let bar = progress.add(
        ProgressBar::new_spinner()
            .with_style(ProgressStyle::with_template("{spinner} {wide_msg}").unwrap()),
    );

    bar.set_message(format!("Searching for {}", path.to_str().unwrap()));
    if path.exists() {
        bar.finish_with_message(format!("{} already downloaded!", path.to_str().unwrap()));
        bar.finish_and_clear();
        return;
    }

    bar.set_message(format!(
        "{} not found! Starting download...",
        path.to_str().unwrap()
    ));
    let response = client.get(url).send().await.unwrap();

    bar.set_message(format!("{url}"));
    bar.set_length(response.content_length().unwrap());
    bar.set_style(
        bar.style()
            .template(
                format!("{{spinner}} {{msg}} {{wide_bar}} {{bytes}}/{{total_bytes}} | {{binary_bytes_per_sec}} | {{duration_precise}}").as_str()
            )
            .unwrap(),
    );

    let temp_file_name = path.file_name().unwrap();
    let temp_path = format!("temp/{}", temp_file_name.to_str().unwrap()).into_boxed_str();
    let temp_file = File::create(temp_path.deref()).await.unwrap();

    let mut writer = BufWriter::new(temp_file);
    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        match item {
            Ok(bytes) => {
                writer.write_all(&bytes).await.unwrap();
                bar.inc(bytes.len().try_into().unwrap());
            }
            Err(e) => {
                bar.abandon_with_message(format!("{e}"));
                remove_file(temp_path.deref()).await.unwrap();
                return;
            }
        }
    }
    writer.flush().await.unwrap();

    bar.set_style(ProgressStyle::with_template("{spinner} {wide_msg}").unwrap());

    if let Err(e) = tokio::fs::rename(temp_path.deref(), path).await {
        bar.abandon_with_message(format!("{e}"));
        remove_file(temp_path.deref()).await.unwrap();
        return;
    }

    bar.finish_with_message("Downloaded file!");
    bar.finish_and_clear();
}
