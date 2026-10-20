use rand::{prelude::*, rng};

const ROM_START: usize = 0x200;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const SCALE: usize = 15;
pub const MEMORY_SIZE: usize = 4 * 1024;

const FONT_START: usize = 0x50;
const FONT: [u8; 80] = [
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

pub struct Instruction {
    opcode: u16,
    t: u8,
    x: u8,
    y: u8,
    n: u8,
    nn: u8,
    nnn: u16,
}

pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    pub screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // only 0 or 1
    pub keyboard: [bool; 16],
    waiting_key_register: Option<usize>,
    state_key: bool,
    stack: [u16; 16],
    stack_pointer: u8,
    program_counter: u16,
    index_register: u16,
    registers: [u8; 16], // chip8 registers
    delay_timer: u8,
}

impl Chip8 {
    pub fn new(filepath: &str) -> Result<Chip8, std::io::Error> {
        let mut chip = Chip8 {
            memory: [0; MEMORY_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            keyboard: [false; 16],
            waiting_key_register: None,
            state_key: false,
            stack: [0; 16],
            stack_pointer: 0,
            program_counter: ROM_START as u16,
            index_register: 0,
            registers: [0; 16],
            delay_timer: 0,
        };

        let rom = std::fs::read(filepath)?;
        if rom.len() > MEMORY_SIZE - ROM_START {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "ROM too large",
            ));
        }
        chip.memory[ROM_START..ROM_START + rom.len()].copy_from_slice(&rom);

        chip.memory[FONT_START..FONT_START + 80].copy_from_slice(&FONT);

        Ok(chip)
    }
}

fn decode(instr: u16) -> Instruction {
    Instruction {
        opcode: instr,
        t: (instr >> 12) as u8,
        x: (instr >> 8 & 0xF) as u8,
        y: (instr >> 4 & 0xF) as u8,
        n: (instr & 0xF) as u8,
        nn: (instr & 0xFF) as u8,
        nnn: instr & 0xFFF,
    }
}

fn draw(chip: &mut Chip8, opcode: &Instruction) {
    chip.registers[0xF] = 0;
    for row in 0..opcode.n {
        let sprite_byte = chip.memory[chip.index_register as usize + row as usize];
        for col in 0..8 {
            let sprite_pixel = (sprite_byte >> (7 - col)) & 0x1;
            if sprite_pixel != 0 {
                let x = (chip.registers[opcode.x as usize] as usize + col as usize) % SCREEN_WIDTH;
                let y = (chip.registers[opcode.y as usize] as usize + row as usize) % SCREEN_HEIGHT;
                let index: usize = y * 64 + x;
                if chip.screen[index] {
                    chip.registers[0xF] = 1;
                }
                chip.screen[index] ^= true;
            }
        }
    }
}

fn execute_instruction(opcode: &Instruction, chip: &mut Chip8) -> Result<(), String> {
    // println!("opcode : {:#X}", opcode.opcode);

    match opcode.t {
        0x0 => {
            if opcode.nn == 0xE0 {
                chip.screen.fill(false);
            } else if opcode.nn == 0xEE {
                if chip.stack_pointer == 0 {
                    return Err("stack underflow".to_string());
                }
                chip.stack_pointer -= 1;
                chip.program_counter = chip.stack[chip.stack_pointer as usize];
            } else {
                return Err(format!("unknown opcode : {:#X}", opcode.opcode));
            }
        }
        0x1 => chip.program_counter = opcode.nnn,
        0x2 => {
            if chip.stack_pointer >= 16 {
                return Err("stack overflow".to_string());
            }
            chip.stack[chip.stack_pointer as usize] = chip.program_counter;
            chip.stack_pointer += 1;
            chip.program_counter = opcode.nnn;
        }
        0x3 => {
            if chip.registers[opcode.x as usize] == opcode.nn {
                chip.program_counter += 2;
            }
        }
        0x4 => {
            if chip.registers[opcode.x as usize] != opcode.nn {
                chip.program_counter += 2;
            }
        }
        0x5 => {
            if chip.registers[opcode.x as usize] == chip.registers[opcode.y as usize] {
                chip.program_counter += 2;
            }
        }
        0x6 => chip.registers[opcode.x as usize] = opcode.nn,
        0x7 => {
            chip.registers[opcode.x as usize] =
                chip.registers[opcode.x as usize].wrapping_add(opcode.nn)
        }
        0x8 => {
            match opcode.n {
                0x0 => chip.registers[opcode.x as usize] = chip.registers[opcode.y as usize],
                0x1 => chip.registers[opcode.x as usize] |= chip.registers[opcode.y as usize],
                0x2 => chip.registers[opcode.x as usize] &= chip.registers[opcode.y as usize],
                0x3 => chip.registers[opcode.x as usize] ^= chip.registers[opcode.y as usize],
                0x4 => {
                    let (result, carry) = chip.registers[opcode.x as usize]
                        .overflowing_add(chip.registers[opcode.y as usize]);
                    chip.registers[opcode.x as usize] = result;
                    chip.registers[0xF] = carry as u8;
                }
                0x5 => {
                    let (result, carry) = chip.registers[opcode.x as usize]
                        .overflowing_sub(chip.registers[opcode.y as usize]);
                    chip.registers[opcode.x as usize] = result;
                    chip.registers[0xF] = !carry as u8;
                }
                0x6 => {
                    let bit = chip.registers[opcode.x as usize] & 0x1;
                    chip.registers[opcode.x as usize] >>= 1;
                    chip.registers[0xF] = bit;
                }
                0x7 => {
                    let (result, carry) = chip.registers[opcode.y as usize]
                        .overflowing_sub(chip.registers[opcode.x as usize]);
                    chip.registers[opcode.x as usize] = result;
                    chip.registers[0xF] = !carry as u8;
                }
                0xE => {
                    chip.registers[0xF] = (chip.registers[opcode.x as usize] & 0x80) >> 7;
                    chip.registers[opcode.x as usize] <<= 1;
                }
                _ => return Err("wrong opcode".to_string()),
            };
        }
        0x9 => {
            if chip.registers[opcode.x as usize] != chip.registers[opcode.y as usize] {
                chip.program_counter += 2;
            }
        }
        0xA => chip.index_register = opcode.nnn,
        0xB => chip.program_counter = opcode.nnn + chip.registers[0] as u16,
        0xC => chip.registers[opcode.x as usize] = rng().random::<u8>() & opcode.nn,
        0xD => draw(chip, opcode),
        0xE => match opcode.nn {
            0x9E => {
                if chip.keyboard[chip.registers[opcode.x as usize] as usize] {
                    chip.program_counter += 2;
                }
            }
            0xA1 => {
                if !chip.keyboard[chip.registers[opcode.x as usize] as usize] {
                    chip.program_counter += 2;
                }
            }
            _ => return Err("wrong opcode".to_string()),
        },
        0xF => match opcode.nn {
            0x07 => chip.registers[opcode.x as usize] = chip.delay_timer,
            0x0A => chip.waiting_key_register = Some(opcode.x as usize),
            0x15 => chip.delay_timer = chip.registers[opcode.x as usize],
            0x18 => {}
            0x1E => chip.index_register += chip.registers[opcode.x as usize] as u16,
            0x29 => chip.index_register = 0x50 + chip.registers[opcode.x as usize] as u16 * 5,
            0x33 => {
                let nbr: u8 = chip.registers[opcode.x as usize];
                chip.memory[chip.index_register as usize] = nbr / 100;
                chip.memory[chip.index_register as usize + 1] = (nbr / 10) % 10;
                chip.memory[chip.index_register as usize + 2] = nbr % 10;
            }
            0x55 => {
                for i in 0..opcode.x + 1 {
                    chip.memory[chip.index_register as usize + i as usize] =
                        chip.registers[i as usize];
                }
            }
            0x65 => {
                for i in 0..opcode.x + 1 {
                    chip.registers[i as usize] =
                        chip.memory[chip.index_register as usize + i as usize];
                }
            }
            _ => return Err("wrong opcode".to_string()),
        },
        _ => unreachable!(), // program panic if we reach this macro
    }

    Ok(())
}

pub fn start_emulator(chip: &mut Chip8) {
    let mut i = 0;

    while i < 10 {
        if let Some(x) = chip.waiting_key_register {
            if let Some(check) = chip.keyboard.iter().position(|y| *y) {
                chip.registers[x] = check as u8;
                chip.state_key = true;
            }
            if chip.state_key && !chip.keyboard[chip.registers[x] as usize] {
                chip.waiting_key_register = None;
                chip.state_key = false;
            }
            break;
        }
        let opcode = decode(u16::from_be_bytes([
            chip.memory[chip.program_counter as usize],
            chip.memory[chip.program_counter as usize + 1],
        ]));
        chip.program_counter += 2;
        execute_instruction(&opcode, chip).unwrap();
        i += 1;
    }

    if chip.delay_timer > 0 {
        chip.delay_timer -= 1;
    }
}
