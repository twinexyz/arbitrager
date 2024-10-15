use regex::Regex;

pub fn is_valid_url(url: &str) -> bool {
    let url_regex = Regex::new(r"^(http|https)://[^\s/$.?#].[^\s]*$").unwrap();
    url_regex.is_match(url)
}
