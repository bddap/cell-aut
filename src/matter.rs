use strum_macros::EnumIter;

use crate::{
    utils::{u32_rgba_to_u8_rgba, u8_rgba_to_u32_rgba},
    EMPTY_COLOR,
};

/// Matter identifier representing matter that we simulate
#[repr(u8)]
#[derive(EnumIter, Debug, Copy, Clone, Eq, PartialEq)]
pub enum MatterId {
    Empty = 0,
    Sand = 1,
    Wood = 2,
}

impl Default for MatterId {
    fn default() -> Self {
        MatterId::Empty
    }
}

impl From<u8> for MatterId {
    fn from(item: u8) -> Self {
        unsafe { std::mem::transmute(item) }
    }
}

impl MatterId {
    fn color_rgba_u8(&self) -> [u8; 4] {
        let color = match *self {
            MatterId::Empty => EMPTY_COLOR,
            MatterId::Sand => 0xc2b280ff,
            MatterId::Wood => 0xba8c63ff,
        };
        u32_rgba_to_u8_rgba(color)
    }

    fn gen_variate_color_rgba_u8(&self) -> [u8; 4] {
        let p: f32 = rand::random();
        let color = self.color_rgba_f32();
        let variation = -0.1 + 0.2 * p;
        let r = ((color[0] + variation).clamp(0.0, 1.0) * 255.0) as u8;
        let g = ((color[1] + variation).clamp(0.0, 1.0) * 255.0) as u8;
        let b = ((color[2] + variation).clamp(0.0, 1.0) * 255.0) as u8;
        let a = 255;
        [r, g, b, a]
    }

    fn color_rgba_f32(&self) -> [f32; 4] {
        let rgba = self.color_rgba_u8();
        [
            rgba[0] as f32 / 255.0,
            rgba[1] as f32 / 255.0,
            rgba[2] as f32 / 255.0,
            rgba[3] as f32 / 255.0,
        ]
    }
}

/// Matter data where first 3 bytes are saved for color and last 4th byte is saved for matter identifier
#[derive(Default, Copy, Clone)]
pub struct MatterWithColor {
    pub value: u32,
}

impl MatterWithColor {
    /// Creates a new matter with color from matter id giving it a slightly randomized color
    pub fn new(matter_id: MatterId) -> MatterWithColor {
        let color = if matter_id != MatterId::Empty {
            matter_id.gen_variate_color_rgba_u8()
        } else {
            matter_id.color_rgba_u8()
        };
        MatterWithColor {
            value: u8_rgba_to_u32_rgba(color[0], color[1], color[2], matter_id as u8),
        }
    }

    pub fn matter_id(&self) -> MatterId {
        ((self.value & 255) as u8).into()
    }
}

impl From<u32> for MatterWithColor {
    fn from(item: u32) -> Self {
        Self { value: item }
    }
}
