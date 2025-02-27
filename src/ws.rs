use serde_json::Value;
use anyhow::{anyhow, Result};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::StreamExt;
use tokio::sync::mpsc;

const WS_URL: &str = "ws://localhost:24050/ws";

#[derive(Debug, Clone, Copy)]
pub enum JudgementChange {
    Great,
    Good,
    Meh,
    Miss,
    None,
    Reset, // if judgements decrease
}

#[derive(Debug, Clone, Copy, Default)]
pub struct JudgementState {
    great: usize,
    good: usize,
    meh: usize,
    miss: usize,
    combo: usize,
}

impl JudgementState {
    // replace with new state and check what changed
    pub fn replace_with(&mut self, other: JudgementState) -> JudgementChange {
        let result: JudgementChange;
        if other.great < self.great || other.good < self.good || other.meh < self.meh || other.miss < self.miss {
            // judgements decreased, reset lights
            result = JudgementChange::Reset;
        } else if other.miss > self.miss {
            // prioritize judgements worst to best
            result = JudgementChange::Miss;
        } else if other.meh > self.meh {
            result = JudgementChange::Meh;
        } else if other.good > self.good {
            result = JudgementChange::Good;
        } else if other.great > self.great {
            result = JudgementChange::Great;
        } else if other.combo != self.combo {
            // probably a slider
            result = JudgementChange::Great;
        } else {
            result = JudgementChange::None;
        }
        *self = other;
        result
    }
}

// convert gosumemory's websocket message to JudgementState
pub async fn parse_state(msg: Message) -> Result<Option<JudgementState>> {
    use tokio_tungstenite::tungstenite::Message::*;
    let text = match msg {
        Text(text) => text,
        Binary(_) => return Err(anyhow!("Binary message received")),
        Close(_) => return Err(anyhow!("Connection closed")),
        _ => return Ok(None),
    };

    let value: Value = serde_json::from_str(&text)?;
    let gameplay = value.get("gameplay").ok_or(anyhow!("No gameplay in websocket message"))?;
    let hits = gameplay.get("hits").ok_or(anyhow!("No hits in websocket message"))?;
    let great = hits.get("300").and_then(|v| v.as_u64()).ok_or(anyhow!("No 300s in websocket message"))? as usize;
    let good = hits.get("100").and_then(|v| v.as_u64()).ok_or(anyhow!("No 100s in websocket message"))? as usize;
    let meh = hits.get("50").and_then(|v| v.as_u64()).ok_or(anyhow!("No 50s in websocket message"))? as usize;
    let miss = hits.get("0").and_then(|v| v.as_u64()).ok_or(anyhow!("No misses in websocket message"))? as usize;

    let combo = gameplay.get("combo").and_then(|v| v.get("current").and_then(|v| v.as_u64())).ok_or(anyhow!("No combo in websocket message"))? as usize;

    Ok(Some(JudgementState {
        great, good, meh, miss, combo
    }))
}

pub async fn connect() -> Result<mpsc::Receiver<Message>> {
    let (tx, rx) = mpsc::channel(32);
    tokio::spawn(async move {
        let stream = connect_async(WS_URL).await.expect("Failed to connect").0;
        let reader = stream.split().1;
        reader.for_each(|msg| async {
            let msg = msg.unwrap();
            tx.send(msg).await.unwrap();
        }).await;
    });
    Ok(rx)
}
