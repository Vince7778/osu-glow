mod ws;
mod keyboard;
mod lights;

use std::time::Duration;

use anyhow::Result;
use keyboard::Keyboard;
use tokio::{select, signal, time};
use wooting_rgb::is_wooting_keyboard_connected;
use ws::JudgementState;

struct AppState {
    keyboard: Keyboard,
    counter: u8,
    judgements: JudgementState,
}

impl AppState {
    fn new() -> Result<Self> {
        Ok(AppState {
            keyboard: Keyboard::new()?,
            counter: 0,
            judgements: JudgementState::default(),
        })
    }
}

// how many updates it takes for the lights to fade out
const FADE_RATE: f32 = 20.0;

async fn update(state: &mut AppState) {
    state.counter = state.counter.wrapping_add(1);

    state.keyboard.update();
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
                    update(&mut state).await;
                } else {
                    println!("Websocket closed, exiting!");
                    break;
                }
            }
            _ = signal.recv() => {
                println!("Received Ctrl-C, exiting!");
                break;
            }
        }
    }

    Ok(())
}
