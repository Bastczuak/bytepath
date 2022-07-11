type RawColor = (u8, u8, u8);

pub const SCREEN_WIDTH: u32 = 800;
pub const SCREEN_HEIGHT: u32 = 600;
pub const SCREEN_RENDER_WIDTH: i32 = 320;
pub const SCREEN_RENDER_HEIGHT: i32 = 200;
pub const RGB_CLEAR_COLOR: RawColor = (16, 16, 16);
pub const RGB_COLOR_BOOST: RawColor = (76, 195, 217);
pub const RGB_COLOR_NON_BOOST: RawColor = (255, 198, 93);
pub const RGB_COLOR_AMMUNITION: RawColor = (123, 200, 164);
pub const RGB_COLOR_DEATH: RawColor = (241, 103, 69);
pub const Z_INDEX_PLAYER: u8 = 10;
pub const Z_INDEX_BOOST_TRAIL: u8 = 20;
pub const SLOW_DOWN_DURATION_ON_DEATH: f32 = 2.5;
