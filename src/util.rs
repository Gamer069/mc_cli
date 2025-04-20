use std::{error::Error, fs::File, io::{Read, Write}, path::Path};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;

pub fn download_text(url: &str, out: &Path, msg: String) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let mut resp = client.get(url).send()?;

    let total = resp.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut f = File::create(out)?;
    let mut buf = [0u8; 8192]; // use a stack array buffer
    let mut downloaded = Vec::new(); // store downloaded data here

    loop {
        let bytes = resp.read(&mut buf)?;
        if bytes == 0 { break; }

        f.write_all(&buf[..bytes])?;
        downloaded.extend_from_slice(&buf[..bytes]);

        pb.inc(bytes as u64);
    }
    pb.finish_with_message(msg);

    Ok(String::from_utf8(downloaded)?) // now return the full text
}

pub fn download_text_no_save(url: &str, msg: String) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let mut resp = client.get(url).send()?;

    let total = resp.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut buf = [0u8; 8192]; // use a stack array buffer
    let mut downloaded = Vec::new(); // store downloaded data here

    loop {
        let bytes = resp.read(&mut buf)?;
        if bytes == 0 { break; }

        downloaded.extend_from_slice(&buf[..bytes]);

        pb.inc(bytes as u64);
    }
    pb.finish_with_message(msg);

    Ok(String::from_utf8(downloaded)?) // now return the full text
}

pub fn download(url: &str, out: &Path, msg: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let client = Client::new();
    let mut resp = client.get(url).send()?;

    let total = resp.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut f = File::create(out)?;
    let mut buf = [0u8; 8192];
    let mut downloaded = Vec::new();

    loop {
        let bytes = resp.read(&mut buf)?;
        if bytes == 0 { break; }

        f.write_all(&buf[..bytes])?;
        downloaded.extend_from_slice(&buf[..bytes]);

        pb.inc(bytes as u64);
    }
    pb.finish_with_message(msg);

    Ok(downloaded)
}

pub fn download_no_save(url: &str, msg: String) -> Result<Vec<u8>, Box<dyn Error>> {
    let client = Client::new();
    let mut resp = client.get(url).send()?;

    let total = resp.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut buf = [0u8; 8192];
    let mut downloaded = Vec::new();

    loop {
        let bytes = resp.read(&mut buf)?;
        if bytes == 0 { break; }

        downloaded.extend_from_slice(&buf[..bytes]);

        pb.inc(bytes as u64);
    }
    pb.finish_with_message(msg);

    Ok(downloaded)
}
