#![allow(clippy::new_without_default)]

pub struct Emulator {
    pub cpu: Cpu,
    pub memory: Memory,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub display: Display,
    pub keys: [bool; 16],
    pub draw_flag: bool,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            memory: Memory::new(),
            delay_timer: 0,
            sound_timer: 0,
            display: Display::new(),
            keys: [false; 16],
            draw_flag: false,
        }
    }

    pub fn init_fontset(&mut self) {
        let fontset: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
            0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
            0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
            0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
            0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];

        for (i, byte) in fontset.iter().enumerate() {
            self.memory.push_data(0x50 + i, *byte);
        }
    }

    pub fn cycle(&mut self) {
        let pc: usize = self.cpu.pc() as usize;
        let memory: &[u8; 4096] = self.memory.data();
        let opcode: u16 = u16::from_be_bytes([memory[pc], memory[pc + 1]]);

        self.execute_opcode(opcode);
        self.update_timers();
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            println!("BEEP");
            self.sound_timer -= 1;
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let data = std::fs::read(path).unwrap();

        for (i, byte) in data.iter().enumerate() {
            self.memory.push_data(0x200 + i, *byte);
        }
    }

    pub fn execute_opcode(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0xF) as usize;
        let y = ((opcode >> 4) & 0xF) as usize;
        let n = (opcode & 0xF) as u8;
        let nn = (opcode & 0xFF) as u8;
        let nnn = opcode & 0xFFF;

        match opcode & 0xF000 {
            0x0000 => match opcode & 0xFF {
                0x00E0 => {
                    let pc = self.cpu.pc();

                    self.display.clear_display();
                    self.cpu.set_pc(pc + 2);
                }

                0x00EE => {
                    let sp = self.cpu.sp();
                    self.cpu.set_sp(sp - 1);

                    let sp = self.cpu.sp();
                    let stack = self.cpu.stack();

                    self.cpu.set_pc(stack[sp as usize] + 2);
                }

                _ => {
                    eprintln!("Unsupported opcode {:#04x}", opcode);
                }
            },

            0x1000 => {
                self.cpu.set_pc(nnn);
            }

            0x2000 => {
                let pc = self.cpu.pc();
                let sp = self.cpu.sp();

                self.cpu.push_stack(pc);
                self.cpu.set_sp(sp + 1);
                self.cpu.set_pc(nnn);
            }

            0x3000 => {
                let v = self.cpu.v();
                let pc = self.cpu.pc();

                if v[x] == (opcode & 0xFF) as u8 {
                    self.cpu.set_pc(pc + 4);
                } else {
                    self.cpu.set_pc(pc + 2);
                }
            }

            0x4000 => {
                let v = self.cpu.v();
                let pc = self.cpu.pc();

                if v[x] != (opcode & 0xFF) as u8 {
                    self.cpu.set_pc(pc + 4);
                } else {
                    self.cpu.set_pc(pc + 2);
                }
            }

            0x5000 => {
                let v = self.cpu.v();
                let pc = self.cpu.pc();

                if v[x] == v[y] {
                    self.cpu.set_pc(pc + 4);
                } else {
                    self.cpu.set_pc(pc + 2);
                }
            }

            0x6000 => {
                let pc = self.cpu.pc();

                self.cpu.push_v(x, nn);
                self.cpu.set_pc(pc + 2);
            }

            0x7000 => {
                let v = self.cpu.v();
                let pc = self.cpu.pc();

                self.cpu.push_v(x, v[x].wrapping_add(nn));
                self.cpu.set_pc(pc + 2);
            }

            0x8000 => {
                let pc = self.cpu.pc();

                match opcode & 0xF {
                    0x0000 => {
                        let v = self.cpu.v();

                        self.cpu.push_v(x, v[y]);
                    }

                    0x0001 => {
                        let v = self.cpu.v();

                        self.cpu.push_v(x, v[x] | v[y]);
                    }

                    0x0002 => {
                        let v = self.cpu.v();

                        self.cpu.push_v(x, v[x] & v[y]);
                    }

                    0x0003 => {
                        let v = self.cpu.v();

                        self.cpu.push_v(x, v[x] ^ v[y]);
                    }

                    0x0004 => {
                        let v = self.cpu.v();
                        let sum = v[x] as u16 + v[y] as u16;

                        if sum > 0xFF {
                            self.cpu.push_v(0xF, 1);
                        } else {
                            self.cpu.push_v(0xF, 0);
                        };

                        self.cpu.push_v(x, (sum & 0xFF) as u8);
                    }

                    0x0005 => {
                        let vx = self.cpu.v()[x];
                        let vy = self.cpu.v()[y];

                        if vx >= vy {
                            self.cpu.push_v(0xF, 1);
                        } else {
                            self.cpu.push_v(0xF, 0);
                        }

                        self.cpu.push_v(x, vx.wrapping_sub(vy));
                    }

                    0x0006 => {
                        let vx = self.cpu.v()[x];

                        self.cpu.push_v(0xF, vx & 0x1);
                        self.cpu.push_v(x, vx >> 1);
                    }

                    0x0007 => {
                        let vx = self.cpu.v()[x];
                        let vy = self.cpu.v()[y];

                        if vy >= vx {
                            self.cpu.push_v(0xF, 1);
                        } else {
                            self.cpu.push_v(0xF, 0);
                        }

                        self.cpu.push_v(x, vy.wrapping_sub(vx));
                    }

                    0x000E => {
                        let vx = self.cpu.v()[x];

                        self.cpu.push_v(0xF, (vx & 0x80) >> 7);
                        self.cpu.push_v(x, vx << 1);
                    }

                    _ => {
                        eprintln!("Unsupported opcode {:#04x}", opcode);
                    }
                }

                self.cpu.set_pc(pc + 2);
            }

            0x9000 => {
                let v = self.cpu.v();
                let pc = self.cpu.pc();

                if v[x] != v[y] {
                    self.cpu.set_pc(pc + 4);
                } else {
                    self.cpu.set_pc(pc + 2);
                }
            }

            0xA000 => {
                let pc = self.cpu.pc();

                self.cpu.set_i(nnn);
                self.cpu.set_pc(pc + 2);
            }

            0xB000 => {
                let v = self.cpu.v();

                self.cpu.set_pc(v[0] as u16 + nnn);
            }

            0xD000 => {
                let memory = self.memory.data();
                let i = self.cpu.i();
                let v = self.cpu.v();
                let pc = self.cpu.pc();
                let vx = v[x];
                let vy = v[y];

                self.cpu.push_v(0xF, 0);

                for row in 0..n {
                    let sprite_byte: u8 = memory[(i + row as u16) as usize];

                    for col in 0..8 {
                        let sprite_pixel = ((sprite_byte >> (7 - col)) & 1) != 0;
                        let screen_x = ((vx.wrapping_add(col)) % 64) as usize;
                        let screen_y = ((vy.wrapping_add(row)) % 32) as usize;

                        let pixel = &mut self.display.pixels[screen_y][screen_x];

                        if sprite_pixel && *pixel {
                            self.cpu.push_v(0xF, 1);
                        }

                        *pixel ^= sprite_pixel;
                    }
                }

                self.draw_flag = true;
                self.cpu.set_pc(pc + 2);
            }

            0xE000 => match opcode & 0xFF {
                0x009E => {
                    let v = self.cpu.v();
                    let pc = self.cpu.pc();

                    if self.keys[v[x] as usize] {
                        self.cpu.set_pc(pc + 4);
                    } else {
                        self.cpu.set_pc(pc + 2);
                    }
                }

                0x00A1 => {
                    let v = self.cpu.v();
                    let pc = self.cpu.pc();

                    if self.keys[v[x] as usize] {
                        self.cpu.set_pc(pc + 2);
                    } else {
                        self.cpu.set_pc(pc + 4);
                    }
                }

                _ => {
                    eprintln!("Unsupported opcode {:#04x}", opcode);
                }
            },

            0xF000 => match opcode & 0xFF {
                0x001E => {
                    let v = self.cpu.v();
                    let i = self.cpu.i();
                    let pc = self.cpu.pc();

                    self.cpu.set_i(i.wrapping_add(v[x] as u16));

                    let i = self.cpu.i();

                    if i > 0xFFF {
                        self.cpu.push_v(0xF, 1);
                    } else {
                        self.cpu.push_v(0xF, 0);
                    }

                    self.cpu.set_pc(pc + 2);
                }

                0x0029 => {
                    let v = self.cpu.v();
                    let pc = self.cpu.pc();

                    self.cpu.set_i(0x50 + (v[x] * 5) as u16);
                    self.cpu.set_pc(pc + 2);
                }

                0x0033 => {
                    let v = self.cpu.v();
                    let value = v[x];
                    let i = self.cpu.i();
                    let pc = self.cpu.pc();

                    self.memory.push_data(i as usize, value / 100);
                    self.memory.push_data((i + 1) as usize, (value / 10) % 10);
                    self.memory.push_data((i + 2) as usize, value % 10);
                    self.cpu.set_pc(pc + 2);
                }

                0x0055 => {
                    let v = self.cpu.v();
                    let i = self.cpu.i();
                    let pc = self.cpu.pc();

                    #[allow(clippy::needless_range_loop)]
                    for index in 0..=x {
                        self.memory.push_data((i + index as u16) as usize, v[index]);
                    }

                    self.cpu.set_pc(pc + 2);
                }

                0x0065 => {
                    let memory = self.memory.data();
                    let i = self.cpu.i();
                    let pc = self.cpu.pc();

                    for index in 0..=x {
                        self.cpu.push_v(index, memory[(i + index as u16) as usize]);
                    }

                    self.cpu.set_pc(pc + 2);
                }

                0x0007 => {
                    let pc = self.cpu.pc();

                    self.cpu.push_v(x, self.delay_timer);
                    self.cpu.set_pc(pc + 2);
                }

                0x0015 => {
                    let v = self.cpu.v();
                    let pc = self.cpu.pc();

                    self.delay_timer = v[x];
                    self.cpu.set_pc(pc + 2);
                }

                0x0018 => {
                    let v = self.cpu.v();
                    let pc = self.cpu.pc();

                    self.sound_timer = v[x];
                    self.cpu.set_pc(pc + 2);
                }

                0x000A => {
                    let mut key_pressed: bool = false;
                    let pc = self.cpu.pc();

                    for i in 0..16 {
                        if self.keys[i] {
                            self.cpu.push_v(x, i as u8);
                            key_pressed = true;
                            break;
                        }
                    }

                    #[allow(clippy::needless_return)]
                    if !key_pressed {
                        return;
                    } else {
                        self.cpu.set_pc(pc + 2);
                    }
                }

                _ => {
                    eprintln!("Unsupported opcode {:#04x}", opcode);
                }
            },

            _ => {
                eprintln!("Unsupported opcode {:#04x}", opcode);
            }
        }
    }
}

pub struct Cpu {
    v: [u8; 16],
    i: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
        }
    }

    pub fn v(&self) -> &[u8; 16] {
        &self.v
    }

    pub fn push_v(&mut self, index: usize, value: u8) {
        self.v[index] = value;
    }

    pub fn i(&self) -> u16 {
        self.i
    }

    pub fn set_i(&mut self, value: u16) {
        self.i = value;
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }

    pub fn stack(&self) -> &[u16; 16] {
        &self.stack
    }

    pub fn push_stack(&mut self, value: u16) {
        self.stack[self.sp as usize] = value;
    }

    pub fn sp(&self) -> u8 {
        self.sp
    }

    pub fn set_sp(&mut self, value: u8) {
        self.sp = value;
    }
}

pub struct Memory {
    data: [u8; 4096],
}

impl Memory {
    pub fn new() -> Self {
        Self { data: [0; 4096] }
    }

    pub fn data(&self) -> &[u8; 4096] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8; 4096] {
        &mut self.data
    }

    pub fn push_data(&mut self, index: usize, value: u8) {
        self.data[index] = value;
    }
}

pub struct Display {
    pixels: [[bool; 64]; 32],
}

impl Display {
    pub fn new() -> Self {
        Self {
            pixels: [[false; 64]; 32],
        }
    }

    pub fn pixels(&self) -> &[[bool; 64]; 32] {
        &self.pixels
    }

    pub fn push_pixels(&mut self, index_y: usize, index_x: usize, value: bool) {
        self.pixels[index_y][index_x] = value;
    }

    pub fn clear_display(&mut self) {
        self.pixels = [[false; 64]; 32];
    }
}
