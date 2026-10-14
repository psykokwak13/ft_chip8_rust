mod chip8;

use crate::chip8::{SCREEN_HEIGHT, SCREEN_WIDTH, chip8_init, start_emulator};
use macroquad::prelude::*;
use std::{env, io, process};

const SCALE: usize = 15;

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

    let mut chip = chip8_init(&argv.next_back().unwrap());

    loop {
        clear_background(BLACK);
        start_emulator(&mut chip);
        next_frame().await;
    }
}
