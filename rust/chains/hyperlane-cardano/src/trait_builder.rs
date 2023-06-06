use url::Url;

/// Cardano connection configuration
#[derive(Debug, Clone)]
pub struct ConnectionConf {
    /// Fully qualified string to connect to
    pub url: Url,
}
