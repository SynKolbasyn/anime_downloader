use std::cmp::Ordering;
use std::io::{stdin, stdout, Write};
use std::process::exit;

use crate::downloader::{download_all_episodes, download_one_episodes, download_range_episodes, DownloadMode};
use crate::parser::{Anime, get_anime_list, parse_all_data, save_data};


#[derive(Debug, PartialEq)]
pub enum UserCommand {
    Error,
    ShowAnimeList,
    DownloadAnime,
    Help,
    Update,
    Exit,
}


pub fn show_menu() {
    print!("[1] - Show anime list\n[2] - Download anime\n[3] - Instructions for use\n[4] - Update cache\n[5] - Exit the program\n~$ ");
    match stdout().flush() {
        Ok(_) => return,
        Err(e) => eprintln!("ERROR: {e}"),
    }
}


pub fn get_user_command() -> UserCommand {
    let mut mode: String = String::new();
    return match stdin().read_line(&mut mode) {
        Ok(_) => {
            match mode.trim().parse::<u8>() {
                Ok(m) => {
                    match m {
                        1 => UserCommand::ShowAnimeList,
                        2 => UserCommand::DownloadAnime,
                        3 => UserCommand::Help,
                        4 => UserCommand::Update,
                        5 => UserCommand::Exit,
                        _ => UserCommand::Error,
                    }
                },
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    UserCommand::Error
                },
            }
        },
        Err(e) => {
            eprintln!("ERROR: {e}");
            UserCommand::Error
        },
    };
}


pub fn execute_command(command: UserCommand) {
    match command {
        UserCommand::ShowAnimeList => show_anime_list(),
        UserCommand::DownloadAnime => download_anime(),
        UserCommand::Help => println!("There should be instructions here"),
        UserCommand::Update => update_data(),
        UserCommand::Exit => exit(0),
        UserCommand::Error => eprintln!("Unknown command"),
    }
    println!("\n--------------------------------------------------\n");
}


pub fn show_anime_list() {
    for (i, e) in get_anime_list().iter().enumerate() {
        println!("{} -> {}", i + 1, e.name);
    }
}


pub fn update_data() {
    save_data(&parse_all_data());
}


pub fn download_anime() {
    let anime_list: Vec<Anime> = get_anime_list();
    if anime_list.is_empty() {
        return;
    }

    show_anime_list();

    print!("\nSelect an anime by entering its number from the list above, or by entering the name\n~$ ");
    stdout().flush().unwrap();

    let mut anime: String = String::new();

    match stdin().read_line(&mut anime) {
        Ok(_) => (),
        Err(e) => eprintln!("ERROR: {e}"),
    }

    match anime.trim().parse::<usize>() {
        Ok(num) => {
            if num < 1 || num > anime_list.len() {
                return;
            }
            let anime: Anime = anime_list[num - 1].clone();
            print!("Did you choose: {}? [Y/n] ", anime.name);
            stdout().flush().unwrap();
            let mut confirmation: String = String::new();
            stdin().read_line(&mut confirmation).unwrap();

            if confirmation.trim().to_lowercase().cmp(&String::from("y")) != Ordering::Equal {
                return;
            }

            print!("[1] - Download one episode\n[2] - Download a range of series\n[3] - Download all episodes\n~$ ");
            stdout().flush().unwrap();
            let mut downloading_mode: String = String::new();
            stdin().read_line(&mut downloading_mode).unwrap();

            let downloading_mode: DownloadMode = match downloading_mode.trim() {
                "1" => DownloadMode::One,
                "2" => DownloadMode::Range,
                "3" => DownloadMode::All,
                _ => DownloadMode::Error,
            };

            match downloading_mode {
                DownloadMode::One => download_one_episodes(&anime.main_page_url),
                DownloadMode::Range => download_range_episodes(&anime.main_page_url),
                DownloadMode::All => download_all_episodes(&anime.main_page_url),
                DownloadMode::Error => println!("Unknown command"),
            };
        },
        Err(_) => {
            println!("{anime}");
            let mut find_list: Vec<Anime> = Vec::new();
            for i in anime_list {
                if i.name.to_lowercase().contains(&anime.trim().to_lowercase()) {
                    find_list.push(i);
                }
            }

            if find_list.is_empty() {
                println!("No matches found");
                return;
            }

            for (i, e) in find_list.iter().enumerate() {
                println!("{} -> {}", i + 1, e.name);
            }
            print!("The following matches were found, select the appropriate one: ");
            stdout().flush().unwrap();
            let mut num: String = String::new();
            stdin().read_line(&mut num).unwrap();
            match num.trim().parse::<usize>() {
                Ok(n) => {
                    if n < 1 || n > find_list.len() {
                        return;
                    }
                    let anime: Anime = find_list[n - 1].clone();
                    print!("Did you choose: {}? [Y/n] ", anime.name);
                    stdout().flush().unwrap();
                    let mut confirmation: String = String::new();
                    stdin().read_line(&mut confirmation).unwrap();

                    if confirmation.trim().to_lowercase().cmp(&String::from("y")) != Ordering::Equal {
                        return;
                    }

                    print!("[1] - Download one episode\n[2] - Download a range of series\n[3] - Download all episodes\n~$ ");
                    stdout().flush().unwrap();
                    let mut downloading_mode: String = String::new();
                    stdin().read_line(&mut downloading_mode).unwrap();

                    let downloading_mode: DownloadMode = match downloading_mode.trim() {
                        "1" => DownloadMode::One,
                        "2" => DownloadMode::Range,
                        "3" => DownloadMode::All,
                        _ => DownloadMode::Error,
                    };

                    match downloading_mode {
                        DownloadMode::One => download_one_episodes(&anime.main_page_url),
                        DownloadMode::Range => download_range_episodes(&anime.main_page_url),
                        DownloadMode::All => download_all_episodes(&anime.main_page_url),
                        DownloadMode::Error => println!("Unknown command"),
                    };
                },
                Err(e) => {
                    println!("ERROR: {e}");
                    return;
                },
            }
        },
    };
}
