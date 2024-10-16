use std::path::Path;
use regex::Regex;

pub fn is_valid_url(url: &str) -> bool {
    let url_regex = Regex::new(r"^(http|https)://[^\s/$.?#].[^\s]*$").unwrap();
    url_regex.is_match(url)
}

pub fn check_directory_exists(path: &str) -> bool {
    let dir_path = Path::new(path);
    dir_path.is_file()
}