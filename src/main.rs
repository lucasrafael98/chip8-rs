mod chip8;
mod fontset;
use std::fs;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("rust-sdl2: chip8", 640, 320)
        .position_centered()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();

    let program = fs::read("./chip8.ch8").expect("Error reading file");
    let mut chip8 = chip8::init(fontset::FONTSET, program, canvas);
    chip8.emu_cycle()
}
