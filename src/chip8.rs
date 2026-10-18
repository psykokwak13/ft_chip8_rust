use std::{fmt::format, fs, io::Read, process};

const ROM_START: usize = 0x200;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const MEMORY_SIZE: usize = 4 * 1024;

pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    keyboard: [bool; 16],
    stack: [u16; 16],
    stack_pointer: u8,
    program_counter: u16,
    index_register: u16,
    registers: [u8; 16], // chip8 registers
    delay_timer: u8,
}

pub struct Instruction {
    opcode: u16,
    t: u8,
    x: u8,
    y: u8,
    n: u8,
    nn: u8,
    nnn: u16,
}

pub fn chip8_init(filepath: &str) -> Chip8 {
    let mut chip = Chip8 {
        memory: [0; MEMORY_SIZE],
        screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
        keyboard: [false; 16],
        stack: [0; 16],
        stack_pointer: 0,
        program_counter: ROM_START as u16,
        index_register: 0,
        registers: [0; 16],
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

    chip
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
        0x7 => chip.registers[opcode.x as usize] += opcode.nn,
        0x8 => {
            match opcode.n {
                0x0 => chip.registers[opcode.x as usize] = chip.registers[opcode.y as usize],
                0x1 => chip.registers[opcode.x as usize] |= chip.registers[opcode.y as usize],
                0x2 => chip.registers[opcode.x as usize] &= chip.registers[opcode.y as usize],
                0x3 => chip.registers[opcode.x as usize] ^= chip.registers[opcode.y as usize],
                0x4 => {
                    if chip.registers[opcode.x as usize] + chip.registers[opcode.y as usize] > 255 {
                        chip.registers[0xF] = 1;
                    } else {
                        chip.registers[0xF] = 0;
                    }
                    chip.registers[opcode.x as usize] += chip.registers[opcode.y as usize];
                }
                0x5 => {
                    if chip.registers[opcode.x as usize] >= chip.registers[opcode.y as usize] {
                        chip.registers[0xF] = 1;
                    } else {
                        chip.registers[0xF] = 0;
                    }
                    chip.registers[opcode.x as usize] -= chip.registers[opcode.y as usize];
                }
                0x6 => {
                    chip.registers[0xF] = chip.registers[opcode.x as usize] & 0x1;
                    chip.registers[opcode.x as usize] >>= 1;
                }
                0x7 => {
                    if chip.registers[opcode.y as usize] >= chip.registers[opcode.x as usize] {
                        chip.registers[0xF] = 1;
                    } else {
                        chip.registers[0xF] = 0;
                    }
                    chip.registers[opcode.x as usize] =
                        chip.registers[opcode.y as usize] - chip.registers[opcode.x as usize];
                }
                0xE => {
                    chip.registers[0xF] = (chip.registers[opcode.x as usize] & 0x80) >> 7;
                    chip.registers[opcode.x as usize] <<= 1;
                }
            };
        }
        0x9 => {
            if chip.registers[opcode.x as usize] != chip.registers[opcode.y as usize] {
                chip.program_counter += 2;
            }
        }
        0xA => chip.index_register = opcode.nnn,
        0xB => ,
        0xC => ,
        0xD => ,
        0xE => ,
        0xF => ,
        _ => unreachable!(), // program panic if we reach this macro
    }

    Ok(())
}

pub fn start_emulator(chip: &mut Chip8) {
    let mut i = 0;

    while i <= 10 {
        let opcode = decode(u16::from_be_bytes([
            chip.memory[chip.program_counter as usize],
            chip.memory[chip.program_counter as usize + 1],
        ]));
        execute_instruction(&opcode, chip);
        chip.program_counter += 2;

        i += 1;
    }
}
