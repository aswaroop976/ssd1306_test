pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub struct Chip8 {
    pub screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT], // 64x32 pixel display
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            screen: [1; SCREEN_WIDTH * SCREEN_HEIGHT],
        };
        chip8
    }
}
