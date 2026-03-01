use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct RawConfig {
    pub(crate) poll_interval_ms: u64,
    pub(crate) actions: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Entry {
    pub(crate) gesture: Vec<String>,
    pub(crate) run: Option<String>,
    pub(crate) cmd: Option<String>,
    pub(crate) keys: Option<Vec<String>>,
}
