use clap::Parser;
use clap_lex::OsStrExt;
use iced::{
    widget::{button, row, text, text_input, Column, Scrollable},
    Color,
};
use regex::bytes::Regex;

use std::{
    error::Error,
    ffi::OsString,
    fs::{self, FileType},
    path::PathBuf,
};

use crate::app::{Message, PADDING};

use super::{accounts::Accounts, button_cell, text_cell, EDGE_PADDING};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Load FILE
    #[arg(long, value_name = "FILE", exclusive = true)]
    load: Option<String>,

    /// Create a new FILE
    #[arg(long, value_name = "FILE", exclusive = true)]
    new: Option<String>,
}

#[derive(Clone, Debug)]
pub struct FilePicker {
    current: PathBuf,
    filename: String,
    error: String,
    show_hidden_files: bool,
}

impl FilePicker {
    pub fn load_or_new_file() -> Option<(Accounts, PathBuf)> {
        let args = Args::parse();

        if let Some(arg) = args.load {
            let path_buf = PathBuf::from(arg);
            let mut accounts = Accounts::load(&path_buf)
                .unwrap_or_else(|err| panic!("error loading {:?}: {}", &path_buf, err));
            accounts.check_monthly();
            accounts.save(&path_buf).unwrap();
            return Some((accounts, path_buf));
        }
        if let Some(arg) = args.new {
            let path_buf = PathBuf::from(arg);
            let accounts = Accounts::new();
            accounts
                .save_first(&path_buf)
                .unwrap_or_else(|err| panic!("error creating {:?}: {}", &path_buf, err));
            return Some((accounts, path_buf));
        }
        None
    }

    pub fn new() -> Self {
        let path = fs::canonicalize(".").unwrap_or_else(|_| PathBuf::from("."));

        Self {
            current: path,
            filename: String::new(),
            error: String::new(),
            show_hidden_files: false,
        }
    }

    pub fn new_file(&mut self, mut file: PathBuf) -> Option<(Accounts, PathBuf)> {
        if file.as_os_str().is_empty() {
            return None;
        }

        let mut file_path = self.current.clone();
        file.set_extension("ron");
        file_path.push(file);
        let accounts = Accounts::new();
        if let Err(err) = accounts.save_first(&file_path) {
            self.error = format!("error creating {:?}: {}", &file_path, err);
            return None;
        }
        Some((accounts, file_path))
    }

    pub fn load_file(&mut self, file_path: &PathBuf) -> Option<Accounts> {
        match Accounts::load(file_path) {
            Ok(mut accounts) => {
                accounts.check_monthly();
                accounts.save(file_path).unwrap();
                Some(accounts)
            }
            Err(err) => {
                self.error = format!("error loading {:?}: {}", &file_path, err);
                None
            }
        }
    }

    pub fn change_dir(&mut self, path_buf: PathBuf) {
        self.current = path_buf;
        self.error = String::new();
    }

    pub fn change_file_name(&mut self, file: &str) {
        self.filename = file.trim().to_string();
        self.error = String::new();
    }

    pub fn view(&self, account: Option<usize>) -> Scrollable<Message> {
        let mut col = Column::new();
        if !self.error.is_empty() {
            col = col.push(text_cell(&self.error));
        }

        if let Some(dir) = self.current.parent() {
            let button = button(text(dir.display())).on_press(Message::ChangeDir(dir.into()));
            col = col.push(button_cell(button));
        }

        col = col.push(text_cell(self.current.to_str_errorless()));

        if account.is_none() {
            let input = text_input("filename", &self.filename)
                .on_input(Message::ChangeFileName)
                .on_submit(Message::NewFile(PathBuf::from(&self.filename)));
            col = col
                .push(row![input, text(".ron"), text(" ".repeat(EDGE_PADDING))].padding(PADDING));

            let is_ron = Regex::new(".ron$").unwrap();
            col = col.push(self.files(&is_ron, account).unwrap());
        } else {
            let is_csv = Regex::new(".csv$").unwrap();
            col = col.push(self.files(&is_csv, account).unwrap());
        }

        col = col.push(button_cell(button("Exit").on_press(Message::Exit)));
        Scrollable::new(col)
    }

    fn files(
        &self,
        file_regex: &Regex,
        account: Option<usize>,
    ) -> Result<Column<Message>, Box<dyn Error>> {
        let mut col = Column::new();
        let mut dirs = Vec::new();
        for entry in fs::read_dir(&self.current)? {
            let dir = entry?;
            dirs.push(dir);
        }
        dirs.sort_by_key(std::fs::DirEntry::file_name);

        for dir in dirs {
            let file_path = dir.path();
            let file_type = dir.file_type()?;
            let file_name = dir.file_name();
            let file_name_str = file_name.to_str_errorless();

            if !self.show_hidden_files && file_name.starts_with(".") {
                continue;
            }

            match file_type_enum(file_type) {
                FileTypeEnum::File => {
                    if file_regex.is_match(file_name.as_encoded_bytes()) {
                        let mut button = button(text(file_name_str))
                            .style(iced::theme::Button::Custom(Box::new(GreenButton)));
                        match account {
                            Some(account) => {
                                button = button.on_press(Message::ImportBoa(account, file_path));
                            }
                            None => button = button.on_press(Message::LoadFile(file_path)),
                        }
                        col = col.push(button_cell(button));
                    }
                }
                FileTypeEnum::Dir => {
                    col = col.push(button_cell(
                        button(text(file_name_str)).on_press(Message::ChangeDir(file_path)),
                    ));
                }
                FileTypeEnum::Symlink => {
                    let file_path_real = fs::read_link(&file_path)?.clone();
                    if let Ok(metadata) = fs::metadata(&file_path) {
                        if metadata.is_file()
                            && file_regex.is_match(file_path_real.as_os_str().as_encoded_bytes())
                        {
                            let s = format!(
                                "{} -> {}",
                                file_name.to_str_errorless(),
                                file_path_real.to_str_errorless(),
                            );

                            let mut button = button(text(&s))
                                .style(iced::theme::Button::Custom(Box::new(GreenButton)));
                            match account {
                                Some(account) => {
                                    button =
                                        button.on_press(Message::ImportBoa(account, file_path));
                                }
                                None => button = button.on_press(Message::LoadFile(file_path)),
                            }
                            col = col.push(button_cell(button));
                        } else if metadata.is_dir() {
                            let s = format!(
                                "{} -> {}",
                                file_name.to_str_errorless(),
                                file_path_real.to_str_errorless(),
                            );
                            col = col.push(button_cell(
                                button(text(&s)).on_press(Message::ChangeDir(file_path)),
                            ));
                        }
                    }
                }
                FileTypeEnum::Unknown => col = col.push(text_cell(file_name_str)),
            }
        }
        Ok(col)
    }

    pub fn show_hidden_files_toggle(&mut self) {
        self.show_hidden_files = !self.show_hidden_files;
    }
}

struct GreenButton;

impl button::StyleSheet for GreenButton {
    type Style = iced::Theme;

    fn active(&self, _: &<Self as button::StyleSheet>::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Color::from_rgb8(0, 255, 0).into()),
            ..Default::default()
        }
    }
}

enum FileTypeEnum {
    Dir,
    File,
    Symlink,
    Unknown,
}

fn file_type_enum(file_type: FileType) -> FileTypeEnum {
    if file_type.is_dir() {
        FileTypeEnum::Dir
    } else if file_type.is_file() {
        FileTypeEnum::File
    } else if file_type.is_symlink() {
        FileTypeEnum::Symlink
    } else {
        FileTypeEnum::Unknown
    }
}

trait ToStrErrorless {
    fn to_str_errorless(&self) -> &str;
}

impl ToStrErrorless for OsString {
    fn to_str_errorless(&self) -> &str {
        self.to_str()
            .map_or("Invalid OsString conversion to &str.", |s| s)
    }
}

impl ToStrErrorless for PathBuf {
    fn to_str_errorless(&self) -> &str {
        self.to_str()
            .map_or("Invalid PathBuf conversion to &str.", |s| s)
    }
}
