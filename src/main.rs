mod ws;
mod keyboard;
mod lights;

use anyhow::Result;
use keyboard::Keyboard;
use tokio::{select, signal};
use wooting_rgb::is_wooting_keyboard_connected;
use ws::JudgementState;

struct AppState {
    keyboard: Keyboard,
    judgements: JudgementState,
}

impl AppState {
    fn new() -> Result<Self> {
        Ok(AppState {
            keyboard: Keyboard::new()?,
            judgements: JudgementState::default(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    if !is_wooting_keyboard_connected() {
        println!("No keyboard found, exiting!");
        return Ok(());
    }

    let mut ws_rx = ws::connect().await?;
    let mut state = AppState::new()?;

    let mut signal = signal::windows::ctrl_c()?;

    loop {
        select! {
            msg = ws_rx.recv() => {
                if let Some(msg) = msg {
                    if let Some(judgements) = ws::parse_state(msg).await? {
                        let change = state.judgements.replace_with(judgements);
                        state.keyboard.read(change);
                    }
                    state.keyboard.update();
                } else {
                    println!("Websocket closed, exiting!");
                    return Ok(());
                }
            }
            _ = signal.recv() => {
                println!("Received Ctrl-C, exiting!");
                return Ok(());
            }
        }
    }
}
