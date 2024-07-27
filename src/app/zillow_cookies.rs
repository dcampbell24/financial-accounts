use std::{fmt, fs};

use chrono::DateTime;
use regex::Regex;
use rusqlite::{Connection, OpenFlags};

#[derive(Debug)]
pub struct Cookie {
    _id: u64,
    _origin_attributes: String,
    name: String,
    value: String,
    pub host: String,
    pub path: String,
    pub expiry: i64,
    _last_accessed: i64,
    _creation_time: i64,
    is_secure: bool,
    is_http_only: bool,
    _in_browser_element: u8,
    same_site: u8,
    _raw_same_site: u8,
    _scheme_map: u8,
    _is_partitioned_attribute_set: bool,
}

impl fmt::Display for Cookie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={};", self.name, self.value)?;
        if self.is_http_only {
            write!(f, " HttpOnly;")?;
        }
        match self.same_site {
            0 => write!(f, " SameSite=None;")?,
            1 => write!(f, " SameSite=Lax;")?,
            2 => write!(f, " SameSite=Strict;")?,
            _ => panic!("You can't have a value greater than 2!"),
        }
        if self.is_secure {
            write!(f, " Secure;")?;
        }
        let expiry = DateTime::from_timestamp(self.expiry, 0).unwrap();
        write!(
            f,
            " Path={}; Domain={}; Expires={};",
            self.path,
            self.host,
            expiry.format("%a, %d %h %Y %H:%M:%S GMT")
        )
    }
}

pub fn get_cookies() -> anyhow::Result<Vec<Cookie>> {
    let mut profile_path = home::home_dir().expect("You don't have a home directory!");
    profile_path.push(".mozilla");
    profile_path.push("firefox");

    let default = Regex::new(r".*default.*")?;
    let mut files = Vec::new();
    for file in fs::read_dir(profile_path)? {
        let file = file?;
        let file_name = file.file_name().into_string().unwrap();
        if default.is_match(&file_name) {
            files.push(file);
        }
    }
    let profile_path = if files.len() == 1 {
        files[0].path()
    } else {
        Err(anyhow::Error::msg(
            r#"You have more than one file with "default" in it."#,
        ))?
    };

    let mut profile_path_default = profile_path.clone();
    profile_path_default.push("cookies.sqlite");
    let mut profile_path_bak = profile_path.clone();
    profile_path_bak.push("cookies.bak.sqlite");

    if let Ok(true) = fs::exists(&profile_path_bak) {
        Err(anyhow::Error::msg(
            r#""cookies.bak.sqlite" already exists!"#,
        ))?;
    } else {
        fs::copy(&profile_path_default, &profile_path_bak)?;
    }

    let conn = Connection::open_with_flags(
        &profile_path_bak,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    let mut stmt = conn.prepare(
        "SELECT \
            id, \
            originAttributes, \
            name, \
            value, \
            host, \
            path, \
            expiry, \
            lastAccessed, \
            creationTime, \
            isSecure, \
            isHttpOnly, \
            inBrowserElement, \
            sameSite, \
            rawSameSite, \
            schemeMap, \
            isPartitionedAttributeSet \
        FROM moz_cookies",
    )?;

    let cookie_iter = stmt.query_map([], |row| {
        Ok(Cookie {
            _id: row.get(0)?,
            _origin_attributes: row.get(1)?,
            name: row.get(2)?,
            value: row.get(3)?,
            host: row.get(4)?,
            path: row.get(5)?,
            expiry: row.get(6)?,
            _last_accessed: row.get(7)?,
            _creation_time: row.get(8)?,
            is_secure: row.get(9)?,
            is_http_only: row.get(10)?,
            _in_browser_element: row.get(11)?,
            same_site: row.get(12)?,
            _raw_same_site: row.get(13)?,
            _scheme_map: row.get(14)?,
            _is_partitioned_attribute_set: row.get(15)?,
        })
    })?;

    let mut cookies = Vec::new();
    let regex = Regex::new(r".*[\.]zillow[\.][[:alpha:]]").unwrap();
    for cookie in cookie_iter {
        if let Ok(cookie) = cookie {
            if regex.is_match(&cookie.host) {
                cookies.push(cookie);
            }
        }
    }

    fs::remove_file(&profile_path_bak)?;
    Ok(cookies)
}
