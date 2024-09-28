use websocket::{stream::sync::NetworkStream, sync::Client, ClientBuilder};
use anyhow::Result;

const WS_URL: &str = "ws://localhost:24050/ws";

/// Interface for the gosumemory websocket
pub struct OsuWebsocket {
    ws: Client<Box<dyn NetworkStream + Send>>,
}

impl OsuWebsocket {
    /// Connect to the websocket
    pub fn connect() -> Result<Self> {
        let ws = ClientBuilder::new(WS_URL)?.connect(None)?;
        Ok(OsuWebsocket { ws })
    }
}
