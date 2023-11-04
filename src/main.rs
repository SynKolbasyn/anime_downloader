use std::io::{stdin, stdout, Write};


#[derive(Debug)]
#[derive(PartialEq)]
enum UserCommand {
    Error,
    ShowAnimeList,
    DownloadAnime,
    Help,
    Exit,
}


fn main() {
    println!("Welcome, this is a program for downloading anime from the site jut.su\n");

    loop {
        show_menu();
        let user_command: UserCommand = get_user_command();
        if user_command == UserCommand::Error {
            println!("Unknown command");
            continue;
        }
        execute_command(user_command);
    }
}


fn show_menu() {
    print!("[1] - Show anime list\n[2] - Download anime\n[3] - Instructions for use\n[4] - Exit the program\n~$ ");
    match stdout().flush() {
        Ok(_) => return,
        Err(e) => eprintln!("ERROR: {e}"),
    }
}


fn get_user_command() -> UserCommand {
    let mut mode = String::new();
    return match stdin().read_line(&mut mode) {
        Ok(_) => {
            match mode.trim().parse::<u8>() {
                Ok(m) => {
                    match m {
                        1 => UserCommand::ShowAnimeList,
                        2 => UserCommand::DownloadAnime,
                        3 => UserCommand::Help,
                        4 => UserCommand::Exit,
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


fn execute_command(command: UserCommand) {
    println!("Executing {:?}...", command);
}
