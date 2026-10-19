mod chip8;

use crate::chip8::{
    Chip8, SCALE, SCREEN_HEIGHT, SCREEN_WIDTH, start_emulator,
};
use macroquad::{
    input::{KeyCode::*, is_key_down},
    prelude::*,
};
use std::{env, io, process};

fn put_on_screen(screen: &[bool; SCREEN_WIDTH * SCREEN_HEIGHT]) {
    for y in 0..32 {
        for x in 0..64 {
            if screen[y * 64 + x] {
                draw_rectangle(
                    x as f32 * SCALE as f32,
                    y as f32 * SCALE as f32,
                    SCALE as f32,
                    SCALE as f32,
                    GREEN,
                );
            }
        }
    }
}


fn handle_key(chip: &mut Chip8) -> (u8, bool) {
    if is_key_down(Key1) {
        chip.keyboard[0x1] = true;
        return (0x1, true);
    }
    if is_key_down(Key2) {
        chip.keyboard[0x2] = true;
        return (0x2, true);
    }
    if is_key_down(Key3) {
        chip.keyboard[0x3] = true;
        return (0x3, true);
    }
    if is_key_down(Key4) {
        chip.keyboard[0xC] = true;
        return (0xC, true);
    }
    if is_key_down(Q) {
        chip.keyboard[0x4] = true;
        return (0x4, true);
    }
    if is_key_down(W) {
        chip.keyboard[0x5] = true;
        return (0x5, true);
    }
    if is_key_down(E) {
        chip.keyboard[0x6] = true;
        return (0x6, true);
    }
    if is_key_down(R) {
        chip.keyboard[0xD] = true;
        return (0xD, true);
    }
    if is_key_down(A) {
        chip.keyboard[0x7] = true;
        return (0x7, true);
    }
    if is_key_down(S) {
        chip.keyboard[0x8] = true;
        return (0x8, true);
    }
    if is_key_down(D) {
        chip.keyboard[0x9] = true;
        return (0x9, true);
    }
    if is_key_down(F) {
        chip.keyboard[0xE] = true;
        return (0xE, true);
    }
    if is_key_down(Z) {
        chip.keyboard[0xA] = true;
        return (0xA, true);
    }
    if is_key_down(X) {
        chip.keyboard[0x0] = true;
        return (0x0, true);
    }
    if is_key_down(C) {
        chip.keyboard[0xB] = true;
        return (0xB, true);
    }
    if is_key_down(V) {
        chip.keyboard[0xF] = true;
        return (0xF, true);
    }

    (0, false)
}

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
        handle_key(&mut chip);
        start_emulator(&mut chip);
        chip.keyboard.fill(false);
        clear_background(BLACK);
        put_on_screen(&chip.screen);
        next_frame().await;
    }
}
