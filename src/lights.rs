use crate::ws::JudgementChange;

const LIGHTS_POS: [(u8, u8); 14] = [(0, 2), (0, 0), (1, 0), (2, 0), (3, 0), (4, 1), (4, 2), (0, 4), (0, 6), (1, 6), (2, 6), (3, 6), (4, 5), (4, 4)];

#[derive(Clone, Copy)]
pub enum LightSide {
    Left,
    Right,
}

pub fn get_lights_from_side(side: impl Into<LightSide>) -> Vec<(u8, u8)> {
    match side.into() {
        LightSide::Left => LIGHTS_POS[..7].to_vec(),
        LightSide::Right => LIGHTS_POS[7..].to_vec(),
    }
}

#[derive(Clone)]
pub struct FadingLight {
    pub side: LightSide,
    time: usize,
    color: (u8, u8, u8),
    fade_rate: f32,
}

impl FadingLight {
    pub fn get_judgement_color(judgement: JudgementChange) -> (u8, u8, u8) {
        match judgement {
            JudgementChange::Great => (80, 80, 120),
            JudgementChange::Good => (0, 200, 0),
            JudgementChange::Meh => (150, 150, 0),
            JudgementChange::Miss => (200, 0, 0),
            _ => (0, 0, 0),
        }
    }

    pub fn new(side: LightSide, color: (u8, u8, u8), fade_rate: f32) -> Self {
        FadingLight {
            side,
            time: u16::MAX as usize, // large default value but won't overflow
            color,
            fade_rate,
        }
    }

    pub fn reset(&mut self) {
        self.time = 0;
    }

    pub fn set_color(&mut self, color: (u8, u8, u8)) {
        self.color = color;
    }

    pub fn update(&mut self) {
        self.time += 1;
    }

    fn get_opacity(&self) -> f32 {
        let t = self.time as f32;
        1.0 - (t / self.fade_rate).min(1.0)
    }

    pub fn get_color(&self) -> (u8, u8, u8) {
        let opacity = self.get_opacity();
        (
            (self.color.0 as f32 * opacity) as u8,
            (self.color.1 as f32 * opacity) as u8,
            (self.color.2 as f32 * opacity) as u8,
        )
    }
}
