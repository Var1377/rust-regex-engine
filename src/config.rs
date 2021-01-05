#[derive(Copy, Clone, Debug)]
pub struct RegexConfig {
    dotall: bool,
    enforce_linear_time_match: bool,
    multithreading: bool,
}

impl Default for RegexConfig {
    fn default() -> Self {
        return RegexConfig {
            dotall: false,
            enforce_linear_time_match: false,
            // Offload utf8 decoding and potentially matching to multiple threads
            multithreading: true,
        };
    }
}
