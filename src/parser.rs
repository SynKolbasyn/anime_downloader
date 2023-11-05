use std::cmp::Ordering;
use std::fs::File;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::{Client, Response};
use serde_json::{json, Value};
use serde::{Serialize, Deserialize};

use crate::downloader::Quality;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Anime {
    pub main_page_url: String,
    pub name: String,
}


#[derive(Debug, Clone)]
pub struct Episode {
    pub url: String,
    pub name: String,
}


pub fn get_all_data(client: &Client) -> String {
    let mut json: String = String::new();
    let mut file = if File::open("cache.json").is_ok() {
        File::options().write(true).read(true).append(false).open("cache.json").unwrap()
    } else {
        let mut f = File::options().write(true).append(false).read(true).create_new(true).open("cache.json").unwrap();
        f.write_all(b"{}").unwrap();
        File::options().write(true).append(false).read(true).open("cache.json").unwrap()
    };
    file.read_to_string(&mut json).unwrap();
    if json.len() == 0 {
        json = String::from("{}");
    }
    let mut json: Value = serde_json::from_str(json.as_str()).unwrap();
    let num_of_pages: u64 = match json["num_of_pages"].as_u64() {
        None => 34,
        _ => json["num_of_pages"].as_u64().unwrap(),
    };

    let bar: ProgressBar = ProgressBar::new(num_of_pages);

    let mut resp = String::new();

    resp.push_str(client.get("https://jut.su/anime/").send().unwrap().text().unwrap().as_str());
    bar.inc(2);

    let mut page: u64 = 2;
    loop {
        let response: Response = client.post("https://jut.su/anime/").header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8").body(format!("ajax_load=yes&start_from_page={page}&show_search=&anime_of_user=")).send().unwrap();

        let text: String = response.text().unwrap();

        if text.cmp(&String::from("empty")) == Ordering::Equal {
            break;
        }

        resp.push_str(text.as_str());

        thread::sleep(Duration::from_millis(250));

        page += 1;
        bar.inc(1);
    }
    bar.finish();

    json["num_of_pages"] = Value::from(page - 1);

    match File::options().write(true).create(true).open("cache.json").unwrap().write_all(json.to_string().as_bytes()) {
        Ok(_) => return resp,
        Err(e) => eprintln!("ERROR: {e}"),
    }

    return resp;
}


pub fn parse_all_data() -> Vec<Anime> {
    let client: Client = Client::builder().user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36").build().unwrap();

    let data: String = get_all_data(&client);
    File::create("site.txt").unwrap().write(data.as_bytes()).unwrap();
    // let mut data: String = String::new();
    // File::open("site.txt").unwrap().read_to_string(&mut data).unwrap();

    let mut anime_list: Vec<Anime> = Vec::new();

    for (i, e) in data.split(r#"<span class="tooltip_pad_in_anime"><a href="/"#).enumerate() {
        if i == 0 {
            continue;
        }
        anime_list.push(Anime{ main_page_url: String::from(format!("https://jut.su/{}/", e.split(r#"/""#).next().unwrap())), name: String::from("???") });
    }

    for (i, e) in data.split(r#"<div class="aaname"#).enumerate() {
        if i == 0 {
            continue;
        }
        anime_list[i - 1].name = String::from(e.split("</div>").next().unwrap())[2..].to_string();
    }

    return anime_list;
}


pub fn save_data(data: &Vec<Anime>) {
    let mut json: String = String::new();
    File::open("cache.json").unwrap().read_to_string(&mut json).unwrap();
    let mut json: Value = serde_json::from_str(json.as_str()).unwrap();
    json["anime_list"] = json!(data);
    serde_json::to_writer(File::create("cache.json").unwrap(), &json).unwrap();
}


pub fn get_anime_list() -> Vec<Anime> {
    let mut json: String = String::new();
    match File::open("cache.json") {
        Ok(mut f) => {
            f.read_to_string(&mut json).unwrap();
        },
        Err(_) => {
            println!("First you should update the cache");
            return Vec::new();
        },
    }
    let json: Value = serde_json::from_str(json.as_str()).unwrap();
    let mut anime_list: Vec<Anime> = Vec::new();
    for i in json["anime_list"].as_array().unwrap() {
        let anime: Anime = serde_json::from_value(i.clone()).unwrap();
        anime_list.push(anime);
    }
    return anime_list;
}


pub fn get_episodes(anime_url: &String, quality: Quality) -> Vec<Episode> {
    let mut episodes: Vec<String> = Vec::new();
    let client: Client = Client::builder().user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36").build().unwrap();
    let tmp: String = client.get(anime_url).send().unwrap().text().unwrap();
    let tmp: Vec<&str> = tmp.split(r#"" class="short-btn"#).collect::<Vec<&str>>();
    let num_of_episodes: usize = tmp.len() - 1;
    for (i, &e) in tmp.iter().enumerate() {
        if i == num_of_episodes {
            break;
        }
        episodes.push(e.split("<a href=").last().unwrap()[1..].to_string());
    }

    return match quality {
        Quality::Ultra => get_1080(&client, &episodes),
        Quality::High => get_720(&client, &episodes),
        Quality::Medium => get_480(&client, &episodes),
        Quality::Low => get_360(&client, &episodes),
        Quality::Error => {
            println!("Unknown quality");
            Vec::new()
        },
    }
}


pub fn get_1080(client: &Client, episodes: &Vec<String>) -> Vec<Episode> {
    let mut res: Vec<Episode> = Vec::new();
    let pb: ProgressBar = ProgressBar::new(episodes.len() as u64);
    pb.set_style(ProgressStyle::default_bar().template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap());
    pb.set_message(format!("Parsing..."));
    for (i, e) in episodes.iter().enumerate() {
        let resp: Response = client.get(format!("https://jut.su{}", e)).send().unwrap();
        let resp_text: String = resp.text().unwrap();

        if resp_text.contains(r#"<div class="block_video_text""#) {
            println!("\nSite access error, VPN connection may be required");
            break;
        }

        let ep_name: String = get_episode_name(&resp_text);

        if !resp_text.contains(r#"res="1080""#) {
            println!("The episode {} does not have the quality of 1080", ep_name);
            continue;
        }

        res.push(Episode {
            url: resp_text.split(r#"<source src=""#).collect::<Vec<&str>>()[1].split(r#"" type="video/mp4""#).next().unwrap().to_string(),
            name: ep_name,
        });
        pb.set_position(i as u64);
    }
    pb.finish();
    return res;
}


pub fn get_720(client: &Client, episodes: &Vec<String>) -> Vec<Episode> {
    let mut res: Vec<Episode> = Vec::new();
    let pb: ProgressBar = ProgressBar::new(episodes.len() as u64);
    pb.set_style(ProgressStyle::default_bar().template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap());
    pb.set_message(format!("Parsing..."));
    for (i, e) in episodes.iter().enumerate() {
        let resp: Response = client.get(format!("https://jut.su{}", e)).send().unwrap();
        let resp_text: String = resp.text().unwrap();

        if resp_text.contains(r#"<div class="block_video_text""#) {
            println!("\nSite access error, VPN connection may be required");
            break;
        }

        let mut index: usize = 2;

        if !resp_text.contains(r#"res="1080""#) {
            index -= 1;
        }

        let ep_name: String = get_episode_name(&resp_text);

        if !resp_text.contains(r#"res="720""#) {
            println!("The episode {} does not have the quality of 720", ep_name);
            continue;
        }

        res.push(Episode {
            url: resp_text.split(r#"<source src=""#).collect::<Vec<&str>>()[index].split(r#"" type="video/mp4""#).next().unwrap().to_string(),
            name: ep_name,
        });
        pb.set_position(i as u64);
    }
    pb.finish();
    return res;
}


pub fn get_480(client: &Client, episodes: &Vec<String>) -> Vec<Episode> {
    let mut res: Vec<Episode> = Vec::new();
    let pb: ProgressBar = ProgressBar::new(episodes.len() as u64);
    pb.set_style(ProgressStyle::default_bar().template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap());
    pb.set_message(format!("Parsing..."));
    for (i, e) in episodes.iter().enumerate() {
        let resp: Response = client.get(format!("https://jut.su{}", e)).send().unwrap();
        let resp_text: String = resp.text().unwrap();

        if resp_text.contains(r#"<div class="block_video_text""#) {
            println!("\nSite access error, VPN connection may be required");
            break;
        }

        let mut index: usize = 3;

        if !resp_text.contains(r#"res="1080""#) {
            index -= 1;
        }

        if !resp_text.contains(r#"res="720""#) {
            index -= 1;
        }

        let ep_name: String = get_episode_name(&resp_text);

        if !resp_text.contains(r#"res="480""#) {
            println!("The episode {} does not have the quality of 480", ep_name);
            continue;
        }

        res.push(Episode {
            url: resp_text.split(r#"<source src=""#).collect::<Vec<&str>>()[index].split(r#"" type="video/mp4""#).next().unwrap().to_string(),
            name: ep_name,
        });
        pb.set_position(i as u64);
    }
    pb.finish();
    return res;
}


pub fn get_360(client: &Client, episodes: &Vec<String>) -> Vec<Episode> {
    let mut res: Vec<Episode> = Vec::new();
    let pb: ProgressBar = ProgressBar::new(episodes.len() as u64);
    pb.set_style(ProgressStyle::default_bar().template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap());
    pb.set_message(format!("Parsing..."));
    for (i, e) in episodes.iter().enumerate() {
        let resp: Response = client.get(format!("https://jut.su{}", e)).send().unwrap();
        let resp_text: String = resp.text().unwrap();

        if resp_text.contains(r#"<div class="block_video_text""#) {
            println!("\nSite access error, VPN connection may be required");
            break;
        }

        let mut index: usize = 4;

        if !resp_text.contains(r#"res="1080""#) {
            index -= 1;
        }

        if !resp_text.contains(r#"res="720""#) {
            index -= 1;
        }

        if !resp_text.contains(r#"res="480""#) {
            index -= 1;
        }

        let ep_name: String = get_episode_name(&resp_text);

        if !resp_text.contains(r#"res="360""#) {
            println!("The episode {} does not have the quality of 360", ep_name);
            continue;
        }

        res.push(Episode {
            url: resp_text.split(r#"<source src=""#).collect::<Vec<&str>>()[index].split(r#"" type="video/mp4""#).next().unwrap().to_string(),
            name: ep_name,
        });
        pb.set_position(i as u64);
    }
    pb.finish();
    return res;
}


pub fn get_episode_name(html: &String) -> String {
    return html.split(r#"<span itemprop="name""#).collect::<Vec<&str>>()[1].split("</span>").next().unwrap().split("</i>").last().unwrap().to_string();
}
