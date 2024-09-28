mod ws;
mod keyboard;
mod lights;

use std::time::Duration;

use anyhow::Result;
use keyboard::Keyboard;
use tokio::{select, signal, time};
use wooting_rgb::is_wooting_keyboard_connected;
use ws::OsuWebsocket;

struct AppState {
    keyboard: Keyboard,
    counter: u8,
    ws: OsuWebsocket,
}

impl AppState {
    fn new(ws: OsuWebsocket) -> Result<Self> {
        Ok(AppState {
            keyboard: Keyboard::new()?,
            counter: 0,
            ws,
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

    let ws = OsuWebsocket::connect()?;
    let mut state = AppState::new(ws)?;

    let mut interval = time::interval(Duration::from_millis(10));
    let mut signal = signal::windows::ctrl_c()?;

    loop {
        select! {
            _ = signal.recv() => {
                println!("Received Ctrl-C, exiting!");
                break;
            }
            _ = interval.tick() => {
                update(&mut state).await;
            }
        }
    }

    Ok(())
}
