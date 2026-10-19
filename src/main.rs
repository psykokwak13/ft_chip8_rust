mod chip8;

use crate::chip8::{Chip8, SCALE, SCREEN_HEIGHT, SCREEN_WIDTH, start_emulator};
use macroquad::prelude::*;
use std::{env, io, process};

#[macroquad::main("chip-8")]
async fn main() -> io::Result<()> {
    let mut argv = env::args();

    if argv.len() != 2 {
        eprintln!("Error : ./chip8 <rom file>");
        process::exit(1);
    }

    // setup gfx
    request_new_screen_size(
        (SCREEN_WIDTH * SCALE) as f32,
        (SCREEN_HEIGHT * SCALE) as f32,
    );

    let mut chip = Chip8::new(&argv.next_back().unwrap())?;

    loop {
        clear_background(BLACK);
        start_emulator(&mut chip);
        next_frame().await;
    }
}
