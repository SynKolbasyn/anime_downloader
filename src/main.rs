pub mod parser;
pub mod ui;
pub mod downloader;


use ui::{show_menu, get_user_command, execute_command};


fn main() {
    println!("Welcome, this is a program for downloading anime from the site jut.su\n");
    loop {
        show_menu();
        execute_command(get_user_command());
    }
}
