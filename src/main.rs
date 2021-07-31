mod chip8;
mod fontset;
use std::fs;

// sdl window scale
const SCALE: u32 = 10;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let events = sdl_context.event_pump().unwrap();
    let window = video_subsystem
        .window("rust-sdl2: chip8", 64 * SCALE, 32 * SCALE)
        .position_centered()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();

    let program = fs::read("./Cave.ch8").expect("Error reading file");
    let mut chip8 = chip8::init(fontset::FONTSET, program, canvas, events);
    chip8.emu_cycle(SCALE as usize);
}
