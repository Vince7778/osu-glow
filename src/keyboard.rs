use wooting_analog_wrapper::ffi::wooting_analog_read_analog;
use wooting_rgb::RgbKeyboard;

use crate::lights::{get_lights_from_side, FadingLight, LightSide};

const PRESS_THRESHOLD: f32 = 0.1;

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

        // TODO: query user for which keys to use
        wooting_analog_wrapper::set_keycode_mode(wooting_analog_wrapper::KeycodeType::VirtualKeyTranslate);
        let keys = (0x5A, 0x43);

        Ok(Keyboard {
            rgb: RgbKeyboard,
            keys,
            keys_pressed: (false, false),
            last_pressed: PressedKey::None,
            lights: (
                FadingLight::new(LightSide::Left, (255, 0, 0), 20.0),
                FadingLight::new(LightSide::Right, (0, 0, 255), 20.0),
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

    pub fn update(&mut self) {
        self.lights.0.update();
        self.lights.1.update();
        self.check_presses();

        // need to clone because you can't borrow self twice
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
        if left_pressed {
            self.lights.0.reset();
        }
        self.keys_pressed.0 = left_pressed;

        if right_pressed && !self.keys_pressed.1 {
            self.last_pressed = PressedKey::Right;
        }
        if right_pressed {
            self.lights.1.reset();
        }
        self.keys_pressed.1 = right_pressed;
    }

    fn write_light(&mut self, light: FadingLight) {
        let color = light.get_color();
        let lights = get_lights_from_side(light.side);
        self.set_all_rgb(lights, color);
    }
}
