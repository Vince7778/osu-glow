use anyhow::Context;
use wooting_analog_wrapper::ffi::wooting_analog_read_analog;
use wooting_rgb::RgbKeyboard;

use crate::{lights::{get_lights_from_side, FadingLight, LightSide}, ws::JudgementChange};

const DEFAULT_COLOR: (u8, u8, u8) = (255, 255, 255);

// how many updates it takes for the lights to fade out
const FADE_RATE: f32 = 10.0;

// distance for analog keys to be considered pressed
const PRESS_THRESHOLD: f32 = 0.1;

const KEY_PATH: &str = "keys.txt";

#[derive(Debug, Clone, Copy)]
pub enum PressedKey {
    Left,
    Right,
    None,
}

pub struct Keyboard {
    rgb: RgbKeyboard,
    keys: (u16, u16),
    keys_pressed: (bool, bool),
    lights: (FadingLight, FadingLight),
    pub last_pressed: PressedKey,
}

impl Keyboard {
    pub fn new() -> anyhow::Result<Self> {
        // Check if a Wooting keyboard is connected
        if !wooting_rgb::is_wooting_keyboard_connected() {
            return Err(anyhow::anyhow!("No Wooting keyboard found!"));
        }

        // Initialize analog SDK
        wooting_analog_wrapper::initialise().0?;

        wooting_analog_wrapper::set_keycode_mode(wooting_analog_wrapper::KeycodeType::VirtualKeyTranslate);

        // read keycodes from file
        // too lazy to make a proper config file
        let key_file = std::fs::read_to_string(KEY_PATH).context("Failed to read keys.txt")?;
        let keys: Vec<u16> = key_file[..2].chars().map(|c| c as u16).collect();
        if keys.len() != 2 {
            return Err(anyhow::anyhow!("keys.txt must contain at least 2 characters"));
        }
        if !keys.iter().all(|k| char::from_u32(*k as u32).unwrap().is_ascii_uppercase()) {
            return Err(anyhow::anyhow!("keys.txt must contain uppercase ascii characters"));
        }

        Ok(Keyboard {
            rgb: RgbKeyboard,
            keys: (keys[0], keys[1]),
            keys_pressed: (false, false),
            last_pressed: PressedKey::None,
            lights: (
                FadingLight::new(LightSide::Left, DEFAULT_COLOR, FADE_RATE),
                FadingLight::new(LightSide::Right, DEFAULT_COLOR, FADE_RATE),
            ),
        })
    }

    pub fn set_rgb(&mut self, pos: (u8, u8), color: (u8, u8, u8)) {
        self.rgb.array_set_single(pos, color.0, color.1, color.2);
    }

    pub fn set_all_rgb(&mut self, pos: Vec<(u8, u8)>, color: (u8, u8, u8)) {
        for p in pos {
            self.set_rgb(p, color);
        }
    }

    pub fn read(&mut self, judgement: JudgementChange) {
        self.check_presses();
        let color = FadingLight::get_judgement_color(judgement);
        match judgement {
            JudgementChange::None => (),
            JudgementChange::Reset => {
                self.lights.0.set_color(color);
                self.lights.1.set_color(color);
            }
            JudgementChange::Miss => {
                self.lights.0.set_color(color);
                self.lights.1.set_color(color);
                self.lights.0.reset();
                self.lights.1.reset();
            }
            _ => {
                let light = match self.last_pressed {
                    PressedKey::Left => &mut self.lights.0,
                    PressedKey::Right => &mut self.lights.1,
                    PressedKey::None => return,
                };
                light.set_color(color);
            }
        }
    }

    pub fn update(&mut self) {
        if self.keys_pressed.0 {
            self.lights.0.reset();
        }
        if self.keys_pressed.1 {
            self.lights.1.reset();
        }
        self.lights.0.update();
        self.lights.1.update();
        self.write_light(self.lights.0.clone());
        self.write_light(self.lights.1.clone());

        self.rgb.array_update();
    }

    fn check_presses(&mut self) {
        // Safety: uhh idk :) hope the keys are valid!
        let left_value = unsafe { wooting_analog_read_analog(self.keys.0) };
        let right_value = unsafe { wooting_analog_read_analog(self.keys.1) };

        let left_pressed = left_value > PRESS_THRESHOLD;
        let right_pressed = right_value > PRESS_THRESHOLD;

        // biases towards right key if both are pressed at the same time
        if left_pressed && !self.keys_pressed.0 {
            self.last_pressed = PressedKey::Left;
        }
        self.keys_pressed.0 = left_pressed;

        if right_pressed && !self.keys_pressed.1 {
            self.last_pressed = PressedKey::Right;
        }
        self.keys_pressed.1 = right_pressed;
    }

    fn write_light(&mut self, light: FadingLight) {
        let color = light.get_color();
        let lights = get_lights_from_side(light.side);
        self.set_all_rgb(lights, color);
    }
}
