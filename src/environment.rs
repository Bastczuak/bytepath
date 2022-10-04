type RawColor = (u8, u8, u8);

pub const SCREEN_RENDER_WIDTH: u32 = 1920 / 2;
pub const SCREEN_RENDER_HEIGHT: u32 = 1080 / 2;
pub const SCREEN_WIDTH: i32 = SCREEN_RENDER_WIDTH as i32 / 2;
pub const SCREEN_HEIGHT: i32 = SCREEN_RENDER_HEIGHT as i32 / 2;
pub const RGB_CLEAR_COLOR: RawColor = (16, 16, 16);
pub const RGB_COLOR_PLAYER: RawColor = (255, 255, 255);
pub const RGB_COLOR_BOOST: RawColor = (76, 195, 217);
pub const RGB_COLOR_TRAIL: RawColor = (255, 198, 93);
// pub const RGB_COLOR_AMMUNITION: RawColor = (123, 200, 164);
pub const RGB_COLOR_DEATH: RawColor = (241, 103, 69);
pub const Z_INDEX_PLAYER: f32 = 10.0;
pub const SLOW_DOWN_DURATION_ON_DEATH: f32 = 2.5;
pub const DEAD_PROJECTILE_WIDTH: f32 = 6.0;
pub const DEAD_PROJECTILE_HEIGHT: f32 = 3.0;
