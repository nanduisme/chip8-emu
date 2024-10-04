use chip8_core::*;
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::{env, fs::File, io::Read};

const SCALE: usize = 10;
const TICKS_PER_FRAME: usize = 10;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    let mut window = Window::new(
        "Chip-8 Emulator",
        SCREEN_WIDTH * SCALE,
        SCREEN_HEIGHT * SCALE,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_target_fps(60);

    let mut chip8 = Emu::new();
    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .get_keys_pressed(KeyRepeat::No)
            .iter()
            .for_each(|key| {
                if let Some(k) = key_to_btn(*key) {
                    chip8.keypress(k, true);
                }
            });

        window.get_keys_released().iter().for_each(|key| {
            if let Some(k) = key_to_btn(*key) {
                chip8.keypress(k, false);
            }
        });

        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }
        chip8.tick_timers();
        window
            .update_with_buffer(
                &get_display_translated(&chip8.get_display()),
                SCREEN_WIDTH * SCALE,
                SCREEN_HEIGHT * SCALE,
            )
            .unwrap();
    }
}

fn get_display_translated(buf: &[bool]) -> [u32; SCREEN_WIDTH * SCREEN_HEIGHT * SCALE * SCALE] {
    let mut stretched = [0u32; SCREEN_WIDTH * SCREEN_HEIGHT * SCALE * SCALE];

    // Loop over the buffer, treating it as a 2D array
    for (index, &value) in buf.iter().enumerate() {
        if value {
            let x = index % SCREEN_WIDTH; // Calculate the x-coordinate
            let y = index / SCREEN_WIDTH; // Calculate the y-coordinate

            // Map (x, y) in the original grid to a SCALE x SCALE block in the stretched grid
            for dy in 0..SCALE {
                for dx in 0..SCALE {
                    let scaled_x = x * SCALE + dx;
                    let scaled_y = y * SCALE + dy;
                    let stretched_index = scaled_y * SCREEN_WIDTH * SCALE + scaled_x;

                    // Set the block to white (0xFFFFFF)
                    stretched[stretched_index] = 0xFFFFFF;
                }
            }
        }
    }

    stretched
}

fn key_to_btn(key: Key) -> Option<usize> {
    match key {
        Key::Key1 => Some(0x1),
        Key::Key2 => Some(0x2),
        Key::Key3 => Some(0x3),
        Key::Key4 => Some(0xc),
        Key::Q => Some(0x4),
        Key::W => Some(0x5),
        Key::E => Some(0x6),
        Key::R => Some(0xD),
        Key::A => Some(0x7),
        Key::S => Some(0x8),
        Key::D => Some(0x9),
        Key::F => Some(0xE),
        Key::Z => Some(0xA),
        Key::X => Some(0x0),
        Key::C => Some(0xB),
        Key::V => Some(0xF),
        _ => None,
    }
}
