#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}

pub struct WebsocketProviderOptions {
    pub connect: bool,
    pub resync_interval: Option<u64>,
    pub max_backoff_time: u64,
}

impl Default for WebsocketProviderOptions {
    fn default() -> Self {
        Self { connect: true, resync_interval: None, max_backoff_time: 2500 }
    }
}
