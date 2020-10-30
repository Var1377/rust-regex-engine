#[derive(Copy, Clone, Debug)]
pub struct RegexConfig {
    pub location: SearchLocation,
    pub case_sensitive: bool,
}

impl Default for RegexConfig {
    fn default() -> Self {
        return RegexConfig {
            case_sensitive: true,
            location: SearchLocation::First,
        };
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SearchLocation {
    Global,
    Sticky(usize),
    First,
}
