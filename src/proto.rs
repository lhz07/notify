//! NOTIFY/1
//! Level: critical
//! Title: disk usage above 90%
//! Author: backup-service
//!
//! /dev/nvme0n1p2 is at 93%.
//! Immediate action required.

use std::{
    borrow::Cow,
    fmt, fs,
    io::{self},
    path::{Path, PathBuf},
    time,
};

use chrono::{DateTime, Local};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Notice,
    Warning,
    Critical,
}

impl Level {
    pub fn as_tui_color(&self) -> ratatui::text::Span<'_> {
        use ratatui::style::Stylize;
        let level_str: &str = self.as_ref();
        match self {
            Level::Info => level_str.blue(),
            Level::Notice => level_str.cyan(),
            Level::Warning => level_str.yellow(),
            Level::Critical => level_str.red().bold(),
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use owo_colors::OwoColorize;

        let level_str: &str = self.as_ref();
        match self {
            Level::Info => write!(f, "{:<8}", level_str.blue()),
            Level::Notice => write!(f, "{:<8}", level_str.cyan()),
            Level::Warning => write!(f, "{:<8}", level_str.yellow()),
            Level::Critical => write!(f, "{:<8}", level_str.red().bold()),
        }
    }
}

impl AsRef<str> for Level {
    fn as_ref(&self) -> &str {
        match self {
            Level::Info => "Info",
            Level::Notice => "Notice",
            Level::Warning => "Warning",
            Level::Critical => "Critical",
        }
    }
}

impl TryFrom<&str> for Level {
    type Error = CatError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let level = match value {
            "Info" => Level::Info,
            "Notice" => Level::Notice,
            "Warning" => Level::Warning,
            "Critical" => Level::Critical,
            _ => return Err(CatError::Invalid("unsupported level".into())),
        };
        Ok(level)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NotifyV1 {
    pub level: Level,
    pub title: String,
    pub program: String,
    pub body: String,
}

#[derive(Debug)]
pub struct Notification {
    /// we may change this to an enum to support
    /// multiple notify versions
    pub notify: NotifyV1,
    pub time: time::SystemTime,
    path: PathBuf,
}

pub trait StrSplit {
    fn length_split(&self, index: usize) -> &str;
}

impl StrSplit for str {
    fn length_split(&self, less_than: usize) -> &str {
        let mut length = 0usize;
        for (i, str) in self.grapheme_indices(true) {
            length += str.width();
            if length > less_than {
                return self.split_at(i).0;
            } else if length == less_than {
                return self.split_at(i + str.len()).0;
            }
        }
        self.as_ref()
    }
}

#[test]
fn test_graph_split() {
    let s = "很长的程序名称";
    println!("s width: {}", s.width());
    for i in 0..=14 {
        println!("\nexpect width: {i}");
        let s = s.length_split(i);
        println!("{}", s);
        println!("actual width: {}", s.width());
        assert!(s.width() <= i);
    }
}

impl fmt::Display for Notification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use chrono::{DateTime, Local};

        let time: DateTime<Local> = DateTime::from(self.time);
        // width: 11
        let time_str = time.format("%m-%d %H:%M");

        // level width: 8
        // space: 2 + 2 = 4
        // total: 11 + 8 + 4 = 23
        // remain: 80 - 23 = 57
        // title: 39
        // space: 2
        // program: 13

        let title = if self.notify.title.width() > 38 {
            let s = format!("{}…", self.notify.title.length_split(38));
            Cow::Owned(s)
        } else {
            Cow::Borrowed(&self.notify.title)
        };

        let program = if self.notify.program.width() > 12 {
            let s = format!("{}…", self.notify.program.length_split(12));
            Cow::Owned(s)
        } else {
            Cow::Borrowed(&self.notify.program)
        };
        write!(
            f,
            "{}  {}  {:<13}  {}",
            time_str, self.notify.level, program, title
        )
    }
}

impl Notification {
    pub fn delete(&self) -> Result<(), io::Error> {
        fs::remove_file(&self.path)
    }
    pub fn time_str(&self) -> String {
        let time: DateTime<Local> = DateTime::from(self.time);
        time.format("%m-%d %H:%M").to_string()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CatError {
    #[error("Invalid: {0}")]
    Invalid(Cow<'static, str>),
    #[error("Incomplete: {0}")]
    Incomplete(Cow<'static, str>),
    #[error("IO: {0}")]
    IO(#[from] std::io::Error),
    #[error("Unexpected: {0}")]
    Unexpected(Cow<'static, str>),
}

impl NotifyV1 {
    pub const VERSION: &str = "NOTIFY/1";
    pub const LEVEL: &str = "Level";
    pub const TITLE: &str = "Title";
    pub const PROGRAM: &str = "Program";
    pub const BODY_LENGTH: &str = "Body-Length";
    pub fn parse(input: &str) -> Result<NotifyV1, CatError> {
        let input = input.trim();
        let (header, body) = input.split_once("\n\n").ok_or(CatError::Invalid(
            "missing an empty line between header and body".into(),
        ))?;
        let body = body.trim();
        let mut lines = header.lines();
        let version = lines.next();
        match version {
            Some(Self::VERSION) => (),
            _ => return Err(CatError::Invalid("version".into())),
        }
        let mut level = None;
        let mut title = None;
        let mut program = None;
        let mut body_length = None;

        while let Some(line) = lines.next()
            && !line.is_empty()
        {
            let (key, val) = line
                .split_once(":")
                .ok_or(CatError::Invalid("header is missing ':'".into()))?;
            let (key, val) = (key.trim(), val.trim());
            match key {
                Self::LEVEL => {
                    let l = Level::try_from(val)?;
                    level = Some(l);
                }
                Self::TITLE => title = Some(val),
                Self::PROGRAM => program = Some(val),
                Self::BODY_LENGTH => {
                    let len: u32 = val
                        .parse()
                        .map_err(|_| CatError::Invalid("body length".into()))?;
                    body_length = Some(len);
                }
                _ => (),
            }
        }
        let level = level.ok_or(CatError::Invalid("missing level".into()))?;
        let title = title
            .ok_or(CatError::Invalid("missing title".into()))?
            .to_string();
        let program = program
            .ok_or(CatError::Invalid("missing program".into()))?
            .to_string();
        let body_length = body_length.ok_or(CatError::Invalid("missing body length".into()))?;
        let body = match body.split_at_checked(body_length as usize) {
            Some((body, _)) => body.to_string(),
            None => {
                return Err(CatError::Incomplete(
                    "body is shorter than Body-Length".into(),
                ));
            }
        };

        let notify = NotifyV1 {
            level,
            title,
            program,
            body,
        };

        Ok(notify)
    }
}

pub fn get_notifications(path: impl AsRef<Path>) -> Result<Vec<Notification>, io::Error> {
    let files = fs::read_dir(&path)
        .map_err(|e| io::Error::new(e.kind(), format!("{e}: {}", path.as_ref().display())))?;
    let mut list = Vec::new();
    for file in files {
        let Ok((time, content, path)) = handle_entry(file) else {
            continue;
        };
        match NotifyV1::parse(&content) {
            Ok(notify) => {
                let notification = Notification { notify, time, path };
                list.push(notification);
            }
            Err(CatError::Incomplete(_)) => {
                let now = time::SystemTime::now();
                match now.duration_since(time) {
                    Ok(dur) => {
                        if dur > time::Duration::from_mins(5) {
                            let _ = fs::remove_file(path);
                        }
                    }
                    Err(e) => {
                        if e.duration() > time::Duration::from_hours(24) {
                            let _ = fs::remove_file(path);
                        }
                    }
                }
            }
            Err(CatError::Invalid(_)) => {
                let _ = fs::remove_file(path);
            }
            _ => (),
        };
    }
    list.sort_by(|a, b| b.time.cmp(&a.time));

    Ok(list)
}

fn handle_entry(
    entry: Result<fs::DirEntry, io::Error>,
) -> Result<(time::SystemTime, String, PathBuf), CatError> {
    let entry = entry?;
    if entry.file_type()?.is_file() {
        let time = entry.metadata()?.modified()?;
        let path = entry.path();
        let content = fs::read_to_string(&path)?;
        Ok((time, content, path))
    } else {
        Err(CatError::Unexpected("expect file".into()))
    }
}

#[test]
fn test_parse() {
    let files = fs::read_dir("test-examples").unwrap();
    for file in files {
        let file = file.unwrap();
        if file.file_name().to_string_lossy().starts_with(".") {
            continue;
        }
        println!("read {}", file.path().display());
        let content = fs::read_to_string(file.path()).unwrap();
        assert!(NotifyV1::parse(&content).is_ok())
    }
}

#[test]
fn test_parse_all() {
    let notifies = get_notifications("test-examples").unwrap();
    println!("{:#?}", notifies);
}
