use macroquad::prelude::*;
use std::{
    env, fs,
    io::{self, Read},
    process,
};

const MEMORY_SIZE: usize = 4 * 1024;
const ROM_START: usize = 0x200;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const SCALE: usize = 15;

struct Nibble {
    opcode: u16,
    t: u8,
    x: u8,
    y: u8,
    n: u8,
    nn: u8,
    nnn: u16,
}

struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    key_event: bool,
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    keyboard: [bool; 16],
    stack: [u16; 16],
    stack_pointer: u8,
    program_counter: u16,
    index_register: u16,
    var_chip8: [u8; 16], // chip8 emulator handle 16 variable
    delay_timer: u8,
}

fn chip8_init(filepath: &str) -> Result<Chip8, &str> {
    let mut chip = Chip8 {
        memory: [0; MEMORY_SIZE],
        key_event: false,
        screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
        keyboard: [false; 16],
        stack: [0; 16],
        stack_pointer: 0,
        program_counter: ROM_START as u16,
        index_register: 0,
        var_chip8: [0; 16],
        delay_timer: 0,
    };

    let mut fd = fs::File::open(filepath).unwrap();
    fd.read(&mut chip.memory[ROM_START..MEMORY_SIZE]).unwrap();

    let font: [u8; 80] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];

    chip.memory[0x50..0x50 + 80].copy_from_slice(&font);

    Ok(chip)
}

#[macroquad::main("chip-8")]
async fn main() -> io::Result<()> {
    let argv = env::args();

    if argv.len() != 2 {
        eprintln!("Error : ./chip8 <rom file>");
        process::exit(1);
    }
    // setup gfx et chip8 emu
    let mut chip = chip8_init(&argv.last().unwrap()).unwrap();
    request_new_screen_size((SCREEN_WIDTH * SCALE) as f32, (SCREEN_HEIGHT * SCALE) as f32);
    clear_background(BLACK);
    loop {
        machine(&chip8);
        next_frame().await;
    }
}
