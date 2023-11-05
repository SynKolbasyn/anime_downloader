use std::cmp::Ordering;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::thread;
use std::thread::JoinHandle;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::blocking::{Response, Client};

use crate::parser::{Episode, get_episodes};

pub enum DownloadMode {
    One,
    All,
    Range,
    Error,
}


pub enum Quality {
    Ultra,
    High,
    Medium,
    Low,
    Error,
}


pub fn download_one_episodes(anime_url: &String) {
    print!("Choose quality:\n[1] - 1080p\n[2] - 720p\n[3] - 480p\n[4] - 360p\n~$ ");
    stdout().flush().unwrap();
    let mut quality: String = String::new();
    stdin().read_line(&mut quality).unwrap();

    let quality: Quality = match quality.trim() {
        "1" => Quality::Ultra,
        "2" => Quality::High,
        "3" => Quality::Medium,
        "4" => Quality::Low,
        _ => Quality::Error,
    };

    let episodes: Vec<Episode> = get_episodes(anime_url, quality);

    if episodes.is_empty() {
        println!("Errors found, try again");
        return;
    }

    let episode: usize = loop {
        for (i, e) in episodes.iter().enumerate() {
            println!("{} -> {}", i + 1, e.name);
        }
        print!("Choose one of the above episodes\n~$ ");
        stdout().flush().unwrap();
        let mut episode: String = String::new();
        stdin().read_line(&mut episode).unwrap();
        match episode.trim().parse() {
            Ok(num) => {
                if num > 0 && num <= episodes.len() {
                    break num;
                }
            },
            Err(e) => println!("ERROR: {e}"),
        }
    };

    let episodes: Vec<Episode> = vec![episodes[episode - 1].clone()];

    start_downloading(&episodes, 1);
}


pub fn download_range_episodes(anime_url: &String) {
    print!("Choose quality:\n[1] - 1080p\n[2] - 720p\n[3] - 480p\n[4] - 360p\n~$ ");
    stdout().flush().unwrap();
    let mut quality: String = String::new();
    stdin().read_line(&mut quality).unwrap();

    let quality: Quality = match quality.trim() {
        "1" => Quality::Ultra,
        "2" => Quality::High,
        "3" => Quality::Medium,
        "4" => Quality::Low,
        _ => Quality::Error,
    };

    let episodes: Vec<Episode> = get_episodes(anime_url, quality);

    if episodes.is_empty() {
        println!("Errors found, try again");
        return;
    }

    let from: usize = loop {
        for (i, e) in episodes.iter().enumerate() {
            println!("{} -> {}", i + 1, e.name);
        }

        print!("Select the start of the range from the list above: ");
        stdout().flush().unwrap();
        let mut from: String = String::new();
        stdin().read_line(&mut from).unwrap();
        match from.trim().parse() {
            Ok(f) => {
                if f > 0 && f < episodes.len() {
                    break f;
                }
            },
            Err(e) => println!("ERROR: {e}"),
        }
    };

    let to: usize = loop {
        for (i, e) in episodes.iter().enumerate() {
            println!("{} -> {}", i + 1, e.name);
        }

        print!("Selected start: {from}\nSelect the end of the range from the list above: ");
        stdout().flush().unwrap();
        let mut to: String = String::new();
        stdin().read_line(&mut to).unwrap();
        match to.trim().parse() {
            Ok(t) => {
                if t > from && t <= episodes.len() {
                    break t;
                }
            },
            Err(e) => println!("ERROR: {e}"),
        }
    };

    let episodes: Vec<Episode> = episodes[from - 1..=to - 1].to_vec();

    start_downloading(&episodes, get_th_count(&episodes));
}


pub fn download_all_episodes(anime_url: &String) {
    print!("Choose quality:\n[1] - 1080p\n[2] - 720p\n[3] - 480p\n[4] - 360p\n~$ ");
    stdout().flush().unwrap();
    let mut quality: String = String::new();
    stdin().read_line(&mut quality).unwrap();

    let quality: Quality = match quality.trim() {
        "1" => Quality::Ultra,
        "2" => Quality::High,
        "3" => Quality::Medium,
        "4" => Quality::Low,
        _ => Quality::Error,
    };

    let episodes: Vec<Episode> = get_episodes(anime_url, quality);

    if episodes.is_empty() {
        println!("Errors found, try again");
        return;
    }

    println!("The following episodes will be downloaded: ");
    for i in &episodes {
        println!("{}", i.name);
    }
    print!("Confirm? [Y/n] ");
    stdout().flush().unwrap();
    let mut confirmation: String = String::new();
    stdin().read_line(&mut confirmation).unwrap();
    if confirmation.trim().to_lowercase().cmp(&String::from("y")) != Ordering::Equal {
        return;
    }

    start_downloading(&episodes, get_th_count(&episodes));
}


pub fn start_downloading(episodes: &Vec<Episode>, th_count: usize) {
    let split: Vec<&[Episode]> = episodes.chunks((episodes.len() as f64 / th_count as f64).ceil() as usize).collect();

    let mut th_handlers: Vec<JoinHandle<()>> = Vec::new();

    let m_pb: MultiProgress = MultiProgress::new();

    for i in split {
        let tmp: Vec<Episode> = i.to_vec();
        let m_pb = m_pb.clone();
        th_handlers.push(thread::spawn(move || {
            let client: Client = Client::builder().user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36").build().unwrap();

            for j in tmp {
                download_episode(&format!("{}.mp4", j.name), &j.url, &client, &m_pb);
            }
        }));
    }

    for handler in th_handlers {
        handler.join().unwrap();
    }

    println!("\nDownload completed");
}


pub fn download_episode(path: &String, url: &String, client: &Client, m_progress_bar: &MultiProgress) {
    let mut resp: Response = client.get(url).send().unwrap();
    let total_size: u64 = resp.content_length().unwrap();
    let pb: ProgressBar = m_progress_bar.add(ProgressBar::new(total_size));
    pb.set_style(ProgressStyle::default_bar().template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap());
    pb.set_message(format!("Downloading {path}"));
    let mut file = pb.wrap_write(File::create(path).unwrap());
    resp.copy_to(&mut file).unwrap();
}


pub fn get_th_count(episodes: &Vec<Episode>) -> usize{
    loop {
        print!("Enter the number of threads to download: ");
        stdout().flush().unwrap();
        let mut th_count: String = String::new();
        stdin().read_line(&mut th_count).unwrap();
        match th_count.trim().parse() {
            Ok(num) => {
                if num > 0 && num <= episodes.len() {
                    return num;
                }
            },
            Err(e) => eprintln!("ERROR: {e}"),
        }
    };
}
