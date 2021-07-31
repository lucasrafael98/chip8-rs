use rand::Rng;
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
    // program counter
    pc: u16,
    sound_timer: u8,
    delay_timer: u8,
    stack: [u16; 16],
    stack_ptr: u16,
    keypad: [bool; 16],
    px_grid: [bool; 64 * 32],
    canvas: Canvas<Window>,
}

pub fn init(fontset: &[u8; 80], program: Vec<u8>, canvas: Canvas<Window>) -> Chip8 {
    let mut chip8 = Chip8 {
        opcode: 0,
        mem: [0; 4096],
        regs: [0; 16],
        addr: 0,
        pc: 0x200,
        sound_timer: 0,
        delay_timer: 0,
        stack: [0; 16],
        stack_ptr: 0,
        keypad: [false; 16],
        px_grid: [false; 64 * 32],
        canvas: canvas,
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
    /// Processes CHIP8 Opcodes.
    /// https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
    fn process_opcode(&mut self) {
        println!("Opcode:\t{:#06x}", self.opcode);
        match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x00FF {
                // clear screen
                0x00E0 => {
                    for i in 0..(64 * 32) {
                        self.px_grid[i] = false;
                    }
                    self.pc += 2;
                }
                // return
                0x00EE => {
                    self.stack_ptr -= 1;
                    self.pc = self.stack[self.stack_ptr as usize];
                    self.pc += 2;
                }
                _ => {
                    println!("WARN: Opcode unknown: {:#06x}", self.opcode);
                }
            },
            // goto
            0x1000 => {
                self.pc = self.opcode & 0x0FFF;
            }
            // call()
            0x2000 => {
                self.stack[self.stack_ptr as usize] = self.pc;
                self.stack_ptr += 1;
                self.pc = self.opcode & 0x0FFF;
            }
            // if vx == nn
            0x3000 => {
                if self.regs[((self.opcode & 0x0F00) >> 8) as usize] == (self.opcode & 0x00FF) as u8
                {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            // if Vx != nn
            0x4000 => {
                if self.regs[((self.opcode & 0x0F00) >> 8) as usize] != (self.opcode & 0x00FF) as u8
                {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            // if vx == vy
            0x5000 => {
                if self.regs[((self.opcode & 0x0F00) >> 8) as usize]
                    == self.regs[((self.opcode & 0x00F0) >> 4) as usize]
                {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            // vx = n
            0x6000 => {
                self.regs[((self.opcode & 0x0F00) >> 8) as usize] = (self.opcode & 0x00FF) as u8;
                self.pc += 2;
            }
            // vx += n
            0x7000 => {
                self.regs[((self.opcode & 0x0F00) >> 8) as usize] += (self.opcode & 0x00FF) as u8;
                self.pc += 2;
            }
            0x8000 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let y = ((self.opcode & 0x00F0) >> 4) as usize;
                match self.opcode & 0x000F {
                    //vx = vy
                    0x0000 => {
                        self.regs[x] = self.regs[y];
                        self.pc += 2;
                    }
                    //vx = vx | vy
                    0x0001 => {
                        self.regs[x] = self.regs[x] | self.regs[y];
                        self.pc += 2;
                    }
                    //vx = vx & vy
                    0x0002 => {
                        self.regs[x] = self.regs[x] & self.regs[y];
                        self.pc += 2;
                    }
                    //vx = vx ^ vy
                    0x0003 => {
                        self.regs[x] = self.regs[x] ^ self.regs[y];
                        self.pc += 2;
                    }
                    //vx += vy
                    0x0004 => {
                        self.regs[x] = self.regs[x] + self.regs[y];
                        self.pc += 2;
                    }
                    //vx -= vy
                    0x0005 => {
                        self.regs[x] = self.regs[x] - self.regs[y];
                        self.pc += 2;
                    }
                    // vx >>= 1, stores most sig bit in vF
                    0x0006 => {
                        self.regs[0xF] = self.regs[x] & 0x1;
                        self.regs[x] >>= 1;
                        self.pc += 2;
                    }
                    // vx = vy - vx
                    0x0007 => {
                        if self.regs[x] > self.regs[y] {
                            self.regs[0xF] = 1;
                        } else {
                            self.regs[0xF] = 0;
                        }
                        self.regs[x] = self.regs[y] - self.regs[x];
                        self.pc += 2;
                    }
                    // vx <<= 1, stores most sig bit in vF
                    0x000E => {
                        self.regs[0xF] = self.regs[x] >> 7;
                        self.regs[x] <<= 1;
                        self.pc += 2;
                    }
                    _ => {
                        println!("WARN: Opcode unknown: {:#06x}", self.opcode);
                    }
                }
            }
            // if vx != vy
            0x9000 => {
                if self.regs[((self.opcode & 0x0F00) >> 8) as usize]
                    != self.regs[((self.opcode & 0x00F0) >> 4) as usize]
                {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            // I = nnn
            0xA000 => {
                self.addr = self.opcode & 0x0FFF;
                self.pc += 2;
            }
            // goto nnn + v0
            0xB000 => {
                self.pc = (self.opcode & 0x0FFF) + self.regs[0x0] as u16;
            }
            // vx = nn + rnd(0,255)
            0xC000 => {
                let mut rng = rand::thread_rng();
                self.regs[((self.opcode & 0x0F00) >> 8) as usize] =
                    rng.gen_range(0..0xFF) + (self.opcode & 0x00FF) as u8;
                self.pc += 2;
            }
            // draw at (vx,vy) sprite 8x(N+1) px
            0xD000 => {
                let x = self.regs[((self.opcode & 0x0F00) >> 8) as usize];
                let y = self.regs[((self.opcode & 0x00F0) >> 4) as usize];
                println!("\tx: {}, y: {}", x, y);
                let height = (self.opcode & 0x000F) as u8;
                let mut px: u16;

                for yline in 0..height {
                    px = self.mem[(self.addr + yline as u16) as usize] as u16;
                    println!("\tPixel:\t{:#06x}", px);
                    for xline in 0..9 {
                        // checking if the (x,y) bit is 1
                        let x_pos = ((x + xline) % 64) as usize;
                        let y_pos = ((y + yline) % 32) as usize;
                        if (px & (0x80 >> xline)) != 0 {
                            if self.px_grid[x_pos + (y_pos * 64)] {
                                self.regs[0xF] = 1;
                            }
                            self.px_grid[x_pos + (y_pos * 64)] ^= true;
                        }
                    }
                }
                self.pc += 2;
            }
            0xE000 => match self.opcode & 0x00FF {
                // if pressed_key == vx
                0x009E => {
                    if self.keypad[self.regs[((self.opcode & 0x0F00) >> 8) as usize] as usize] {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                }
                // if pressed_key != vx
                0x00A1 => {
                    if !self.keypad[self.regs[((self.opcode & 0x0F00) >> 8) as usize] as usize] {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                }
                _ => {
                    println!("WARN: Opcode unknown: {:#06x}", self.opcode);
                }
            },
            0xF000 => match self.opcode & 0x00FF {
                // vx = delay
                0x0007 => {
                    self.regs[((self.opcode & 0x0F00) >> 8) as usize] = self.delay_timer;
                    self.pc += 2;
                }
                // vx = wait_for_key() - blocks program
                0x000A => {
                    let mut key_pressed = false;
                    for (i, key) in self.keypad.iter().enumerate() {
                        if *key {
                            self.regs[((self.opcode & 0x0F00) >> 8) as usize] = i as u8;
                            key_pressed = true;
                            break;
                        }
                    }

                    if !key_pressed {
                        return; // without increasing PC, keeps waiting for key
                    }
                    self.pc += 2;
                }
                // delay_timer = vx
                0x0015 => {
                    self.delay_timer = self.regs[((self.opcode & 0x0F00) >> 8) as usize];
                    self.pc += 2;
                }
                // sound_timer = vx
                0x0018 => {
                    self.sound_timer = self.regs[((self.opcode & 0x0F00) >> 8) as usize];
                    self.pc += 2;
                }
                // I += vx
                0x001E => {
                    self.addr += self.regs[((self.opcode & 0x0F00) >> 8) as usize] as u16;
                    self.pc += 2;
                }
                // I = sprite_addr[vx]
                0x0029 => {
                    self.addr = (self.regs[((self.opcode & 0x0F00) >> 8) as usize] * 0x5) as u16;
                    self.pc += 2;
                }
                // store binary-coded decimal vx at I, I+1, I+2
                0x0033 => {
                    self.mem[self.addr as usize] =
                        self.regs[((self.opcode & 0x0F00) >> 8) as usize] / 100;
                    self.mem[(self.addr + 1) as usize] =
                        self.regs[((self.opcode & 0x0F00) >> 8) as usize] / 10 % 10;
                    self.mem[(self.addr + 2) as usize] =
                        self.regs[((self.opcode & 0x0F00) >> 8) as usize] / 100 % 10;
                    self.pc += 2;
                }
                // save v0-x in mem starting at I
                0x0055 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    for i in 0..(x + 1) {
                        self.mem[self.addr as usize + 1] = self.regs[i as usize];
                    }
                    self.addr += x + 1;
                    self.pc += 2;
                }
                // load v0-x from mem starting at I
                0x0065 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    for i in 0..(x + 1) {
                        self.regs[i as usize] = self.mem[self.addr as usize + 1];
                    }
                    self.addr += x + 1;
                    self.pc += 2;
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

    pub fn emu_cycle(&mut self) {
        loop {
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

            self.canvas.set_draw_color(Color::RGB(255, 255, 255));
            let mut to_draw = Vec::new();

            for x in 0..64 {
                for y in 0..32 {
                    if self.px_grid[x + y * 64] {
                        for x1 in 0..10 {
                            for y1 in 0..10 {
                                to_draw
                                    .push(Point::from(((x * 10 + x1) as i32, (y * 10 + y1) as i32)))
                            }
                        }
                    }
                }
            }

            self.canvas
                .draw_points(&to_draw[..])
                .expect("Error drawing!");

            self.canvas.present();
            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
