pub struct Config {
    tag_format: String,
}

impl Config {
    pub fn new() -> Self {
        Config::default()
    }

    pub fn tag_format(&self) -> &str {
        &self.tag_format
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            tag_format: String::from("v{version}"),
        }
    }
}
