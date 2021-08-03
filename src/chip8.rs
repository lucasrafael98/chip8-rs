use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;

pub struct Chip8 {
    opcode: u16,
    mem: [u8; 4096],
    // vx = registers
    regs: [u8; 16],
    // usually called I
    addr: u16,
    pc: u16,
    sound_timer: u8,
    delay_timer: u8,
    stack: [u16; 16],
    sp: u16,
    px_grid: [bool; 64 * 32],
    canvas: Canvas<Window>,
    keypad: [bool; 16],
    events: sdl2::EventPump,
    redraw: bool,
}

pub fn init(
    fontset: &[u8; 80],
    program: Vec<u8>,
    canvas: Canvas<Window>,
    events: sdl2::EventPump,
) -> Chip8 {
    let mut chip8 = Chip8 {
        opcode: 0,
        mem: [0; 4096],
        regs: [0; 16],
        addr: 0,
        pc: 0x200,
        sound_timer: 0,
        delay_timer: 0,
        stack: [0; 16],
        sp: 0,
        px_grid: [false; 64 * 32],
        canvas: canvas,
        keypad: [false; 16],
        events: events,
        redraw: false,
    };
    for (i, _) in fontset.iter().enumerate() {
        chip8.mem[i] = fontset[i];
    }

    for (i, _) in program.iter().enumerate() {
        chip8.mem[i + 512] = program[i];
    }

    return chip8;
}

impl Chip8 {
    pub fn emu_cycle(&mut self, draw_scale: usize, speed: u32) {
        loop {
            self.redraw = false;

            self.handle_keys();
            if self
                .events
                .keyboard_state()
                .is_scancode_pressed(Scancode::Escape)
            {
                println!("Quitting...");
                return;
            }

            let pc = self.pc as usize;
            self.opcode = (self.mem[pc] as u16) << 8 | (self.mem[pc + 1] as u16);
            self.process_opcode();

            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                if self.sound_timer == 1 {
                    println!("beep!");
                }
                self.sound_timer -= 1;
            }

            if self.redraw {
                self.draw_canvas(draw_scale);
            }
            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / (60 * speed)));
        }
    }

    fn draw_canvas(&mut self, draw_scale: usize) {
        let mut to_draw_white = Vec::new();
        let mut to_draw_black = Vec::new();

        for x in 0..64 {
            for y in 0..32 {
                if self.px_grid[x + y * 64] {
                    for x1 in 0..draw_scale {
                        for y1 in 0..draw_scale {
                            to_draw_white.push(Point::from((
                                (x * draw_scale + x1) as i32,
                                (y * draw_scale + y1) as i32,
                            )))
                        }
                    }
                } else {
                    for x1 in 0..draw_scale {
                        for y1 in 0..draw_scale {
                            to_draw_black.push(Point::from((
                                (x * draw_scale + x1) as i32,
                                (y * draw_scale + y1) as i32,
                            )))
                        }
                    }
                }
            }
        }
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        self.canvas
            .draw_points(&to_draw_white[..])
            .expect("Error drawing!");

        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas
            .draw_points(&to_draw_black[..])
            .expect("Error drawing!");

        self.canvas.present();
    }

    /// Processes CHIP8 Opcodes.
    /// https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
    fn process_opcode(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as usize;
        let y = ((self.opcode & 0x00F0) >> 4) as usize;
        let vx = self.regs[x];
        let vy = self.regs[y];
        let nnn = self.opcode & 0x0FFF;
        let nn = (self.opcode & 0x00FF) as u8;
        let n = (self.opcode & 0x000F) as u8;

        self.pc += 2;

        match self.opcode & 0xF000 {
            0x0000 => match nn {
                // clear screen
                0x00E0 => {
                    for i in 0..(64 * 32) {
                        self.px_grid[i] = false;
                    }
                }
                // return
                0x00EE => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                }
                _ => {
                    println!("WARN: Opcode unknown: {:#06x}", self.opcode);
                }
            },
            // goto nnn
            0x1000 => {
                self.pc = nnn;
            }
            // call()
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }
            // if vx == nn
            0x3000 => {
                if vx == nn {
                    self.pc += 2;
                }
            }
            // if Vx != nn
            0x4000 => {
                if vx != nn {
                    self.pc += 2;
                }
            }
            // if vx == vy
            0x5000 => {
                if vx == vy {
                    self.pc += 2;
                }
            }
            // vx = n
            0x6000 => {
                self.regs[x] = nn;
            }
            // vx += n
            0x7000 => {
                self.regs[x] += n;
            }
            0x8000 => {
                match n {
                    //vx = vy
                    0x0000 => {
                        self.regs[x] = self.regs[y];
                    }
                    //vx = vx | vy
                    0x0001 => {
                        self.regs[x] = self.regs[x] | self.regs[y];
                    }
                    //vx = vx & vy
                    0x0002 => {
                        self.regs[x] = self.regs[x] & self.regs[y];
                    }
                    //vx = vx ^ vy
                    0x0003 => {
                        self.regs[x] = self.regs[x] ^ self.regs[y];
                    }
                    //vx += vy
                    0x0004 => {
                        let (res, overflow) = self.regs[x].overflowing_add(self.regs[y]);
                        self.regs[0xF] = if overflow { 1 } else { 0 };
                        self.regs[x] = res;
                    }
                    //vx -= vy
                    0x0005 => {
                        let (res, overflow) = self.regs[x].overflowing_sub(self.regs[y]);
                        self.regs[0xF] = if overflow { 1 } else { 0 };
                        self.regs[x] = res;
                    }
                    // vx >>= 1, stores most sig bit in vF
                    0x0006 => {
                        self.regs[0xF] = self.regs[x] & 0x1;
                        self.regs[x] >>= 1;
                    }
                    // vx = vy - vx
                    0x0007 => {
                        let (res, overflow) = self.regs[y].overflowing_sub(self.regs[x]);
                        self.regs[0xF] = if overflow { 1 } else { 0 };
                        self.regs[x] = res;
                    }
                    // vx <<= 1, stores most sig bit in vF
                    0x000E => {
                        self.regs[0xF] = self.regs[x] >> 7;
                        self.regs[x] <<= 1;
                    }
                    _ => {
                        println!("WARN: Opcode unknown: {:#06x}", self.opcode);
                    }
                }
            }
            // if vx != vy
            0x9000 => {
                if vx != vy {
                    self.pc += 2;
                }
            }
            // I = nnn
            0xA000 => {
                self.addr = nnn;
            }
            // goto nnn + v0
            0xB000 => {
                self.pc = nnn + self.regs[0] as u16;
            }
            // vx = rnd(0,255) & nn
            0xC000 => {
                let mut rng = rand::thread_rng();
                self.regs[x] = rng.gen_range(0..0xFF) & nn;
            }
            // draw at (vx,vy) sprite 8x(N+1) px
            0xD000 => {
                let mut px: u16;
                self.regs[0xF] = 0;

                for yline in 0..n {
                    px = self.mem[(self.addr + yline as u16) as usize] as u16;
                    for xline in 0..8 {
                        // checking if the (x,y) bit is 1
                        let x_pos = ((vx + xline) % 64) as usize;
                        let y_pos = ((vy + yline) % 32) as usize;

                        let color = (px >> (7 - xline as u16)) & 1;
                        if color != 0 {
                            self.regs[0xF] |=
                                (color & self.px_grid[x_pos + (y_pos * 64)] as u16) as u8;
                        }
                        self.px_grid[x_pos + (y_pos * 64)] ^= color != 0;
                    }
                }
                self.redraw = true;
            }
            0xE000 => match nn {
                // if pressed_key == vx
                0x009E => {
                    if self.keypad[vx as usize] {
                        self.pc += 2;
                    }
                }
                // if pressed_key != vx
                0x00A1 => {
                    if !self.keypad[vx as usize] {
                        self.pc += 2;
                    }
                }
                _ => {
                    println!("WARN: Opcode unknown: {:#06x}", self.opcode);
                }
            },
            0xF000 => match nn {
                // vx = delay
                0x0007 => {
                    self.regs[x] = self.delay_timer;
                }
                // vx = wait_for_key() - blocks program
                0x000A => {
                    let mut key_pressed = false;
                    for (i, key) in self.keypad.iter().enumerate() {
                        if *key {
                            self.regs[x] = i as u8;
                            key_pressed = true;
                            break;
                        }
                    }

                    if !key_pressed {
                        self.pc -= 2;
                    }
                }
                // delay_timer = vx
                0x0015 => {
                    self.delay_timer = vx;
                }
                // sound_timer = vx
                0x0018 => {
                    self.sound_timer = vx;
                }
                // I += vx
                0x001E => {
                    self.addr += vx as u16;
                }
                // I = sprite_addr[vx]
                0x0029 => {
                    self.addr = (vx * 0x5) as u16;
                }
                // store binary-coded decimal vx at I, I+1, I+2
                0x0033 => {
                    self.mem[self.addr as usize] = vx / 100;
                    self.mem[(self.addr + 1) as usize] = vx / 10 % 10;
                    self.mem[(self.addr + 2) as usize] = vx / 100 % 10;
                }
                // save v0-x in mem starting at I
                0x0055 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    for i in 0..(x + 1) {
                        self.mem[self.addr as usize + 1] = self.regs[i as usize];
                    }
                    self.addr += x + 1;
                }
                // load v0-x from mem starting at I
                0x0065 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    for i in 0..(x + 1) {
                        self.regs[i as usize] = self.mem[self.addr as usize + 1];
                    }
                    self.addr += x + 1;
                }
                _ => {
                    println!("WARN: Opcode unknown: {:#06x}", self.opcode);
                }
            },
            _ => {
                println!("WARN: Opcode unknown: {:#06x}", self.opcode);
            }
        }
    }

    fn handle_keys(&mut self) {
        for event in self.events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => {
                    self.keypad[0] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => {
                    self.keypad[1] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => {
                    self.keypad[2] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    ..
                } => {
                    self.keypad[3] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => {
                    self.keypad[4] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    self.keypad[5] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => {
                    self.keypad[6] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    self.keypad[7] = true;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    self.keypad[8] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    self.keypad[9] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    self.keypad[10] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => {
                    self.keypad[11] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    ..
                } => {
                    self.keypad[12] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => {
                    self.keypad[13] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => {
                    self.keypad[14] = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::V),
                    ..
                } => {
                    self.keypad[15] = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Num1),
                    ..
                } => {
                    self.keypad[0] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Num2),
                    ..
                } => {
                    self.keypad[1] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Num3),
                    ..
                } => {
                    self.keypad[2] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Num4),
                    ..
                } => {
                    self.keypad[3] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Q),
                    ..
                } => {
                    self.keypad[4] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    self.keypad[5] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::E),
                    ..
                } => {
                    self.keypad[6] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    self.keypad[7] = false;
                }

                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    self.keypad[8] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    self.keypad[9] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    self.keypad[10] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::F),
                    ..
                } => {
                    self.keypad[11] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Z),
                    ..
                } => {
                    self.keypad[12] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::X),
                    ..
                } => {
                    self.keypad[13] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::C),
                    ..
                } => {
                    self.keypad[14] = false;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::V),
                    ..
                } => {
                    self.keypad[15] = false;
                }
                _ => {}
            }
        }
    }
}
