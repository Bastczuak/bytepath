type RawColor = (u8, u8, u8);

pub const SCREEN_RENDER_WIDTH: u32 = 1920 / 2;
pub const SCREEN_RENDER_HEIGHT: u32 = 1080 / 2;
pub const SCREEN_WIDTH: i32 = SCREEN_RENDER_WIDTH as i32 / 2;
pub const SCREEN_HEIGHT: i32 = SCREEN_RENDER_HEIGHT as i32 / 2;
pub const RGB_CLEAR_COLOR: RawColor = (16, 16, 16);
pub const RGB_COLOR_PLAYER: RawColor = (255, 255, 255);
// pub const RGB_COLOR_BOOST: RawColor = (76, 195, 217);
// pub const RGB_COLOR_NON_BOOST: RawColor = (255, 198, 93);
// pub const RGB_COLOR_AMMUNITION: RawColor = (123, 200, 164);
// pub const RGB_COLOR_DEATH: RawColor = (241, 103, 69);
// pub const Z_INDEX_PLAYER: u8 = 10;
// pub const Z_INDEX_BOOST_TRAIL: u8 = 20;
// pub const SLOW_DOWN_DURATION_ON_DEATH: f32 = 2.5;
