mod chip8;

use crate::chip8::{Chip8, SCALE, SCREEN_HEIGHT, SCREEN_WIDTH, start_emulator};
use macroquad::{input::KeyCode::*, prelude::*};

fn put_on_screen(screen: &[bool; SCREEN_WIDTH * SCREEN_HEIGHT]) {
    for y in 0..32 {
        for x in 0..64 {
            if screen[y * 64 + x] {
                draw_rectangle(
                    x as f32 * SCALE as f32,
                    y as f32 * SCALE as f32,
                    SCALE as f32,
                    SCALE as f32,
                    ORANGE,
                );
            }
        }
    }
}

const KEYMAP: [KeyCode; 16] = [
    X,    // 0x0
    Key1, // 0x1
    Key2, // 0x2
    Key3, // 0x3
    Q,    // 0x4
    W,    // 0x5
    E,    // 0x6
    A,    // 0x7
    S,    // 0x8
    D,    // 0x9
    Z,    // 0xA
    C,    // 0xB
    Key4, // 0xC
    R,    // 0xD
    F,    // 0xE
    V,    // 0xF
];

fn handle_key(chip: &mut Chip8) {
    for (i, key) in KEYMAP.iter().enumerate() {
        chip.keyboard[i] = is_key_down(*key);
    }
}

#[macroquad::main("chip-8")]
async fn main() -> std::io::Result<()> {
    let mut argv = std::env::args();

    if argv.len() != 2 {
        eprintln!("Error : ./chip8 <rom file>");
        std::process::exit(1);
    }

    // setup gfx
    request_new_screen_size(
        (SCREEN_WIDTH * SCALE) as f32,
        (SCREEN_HEIGHT * SCALE) as f32,
    );

    let mut chip = Chip8::new(&argv.next_back().unwrap())?;

    loop {
        if is_key_pressed(Escape) {
            break;
        }

        handle_key(&mut chip);
        start_emulator(&mut chip);
        chip.keyboard.fill(false);
        clear_background(BLACK);
        put_on_screen(&chip.screen);
        next_frame().await;
    }
    Ok(())
}
