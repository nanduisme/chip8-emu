use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const NUM_KEYS: usize = 16;
const STACK_SIZE: usize = 16;

const START_ADDR: usize = 0x200;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],

    dt: u8,
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: 0x200,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR as u16;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        let op = self.fetch();

        let d1 = (op & 0xf000) >> 12;
        let d2 = (op & 0x0f00) >> 8;
        let d3 = (op & 0x00f0) >> 4;
        let d4 = op & 0x000f;

        match (d1 as usize, d2 as usize, d3 as usize, d4 as usize) {
            // NO-OP
            (0, 0, 0, 0) => (),

            // Clear screen
            (0, 0, 0xe, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],

            // Return from subroutine
            (0, 0, 0xe, 0xe) => {
                let ret = self.pop();
                self.pc = ret;
            }

            // Jump NNN
            (1, _, _, _) => {
                let nnn = op & 0x0fff;
                self.pc = nnn;
            }

            // Call NNN
            (2, _, _, _) => {
                let nnn = op & 0x0fff;
                self.push(self.pc);
                self.pc = nnn;
            }

            // 3XNN Skip next if VX == NN
            (3, x, _, _) => {
                let nn = (op & 0x00ff) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }

            // 4XNN Skip if VX != NN
            (4, x, _, _) => {
                let nn = (op & 0x00ff) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }

            // 5XY0 - Skip next if VX == VY
            (5, x, y, 0) => {
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // 6XNN - VX = NN
            (6, x, _, _) => {
                let nn = (op & 0x00ff) as u8;
                self.v_reg[x] = nn;
            }

            // 7XNN - VX += NN
            (7, x, _, _) => {
                let nn = (op & 0x00ff) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }

            // 8XY0 Load VY into VX
            (8, x, y, 0) => self.v_reg[x] = self.v_reg[y],

            // 8XY1 VX |= VY
            (8, x, y, 1) => self.v_reg[x] |= self.v_reg[y],

            // 8XY2 VX &= VY
            (8, x, y, 2) => self.v_reg[x] &= self.v_reg[y],

            // 8XY3 VX ^= VY
            (8, x, y, 3) => self.v_reg[x] ^= self.v_reg[y],

            // 8XY4 VX += VY
            (8, x, y, 4) => {
                let overflown;
                (self.v_reg[x], overflown) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                self.v_reg[0xf] = overflown as u8;
            }

            // 8XY5 VX -= VY
            (8, x, y, 5) => {
                let underflown;
                (self.v_reg[x], underflown) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                self.v_reg[0xf] = !underflown as u8;
            }

            // 8X_6 VX >>= 1
            (8, x, _, 6) => {
                self.v_reg[0xf] = self.v_reg[x] & 0x1;
                self.v_reg[x] >>= 1;
            }

            // 8XY7 VX = VY - VX
            (8, x, y, 7) => {
                let underflown;
                (self.v_reg[x], underflown) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                self.v_reg[0xf] = !underflown as u8;
            }

            // 8X_E VX <<= 1
            (8, x, _, 0xe) => {
                self.v_reg[0xf] = self.v_reg[x] >> 7;
                self.v_reg[x] <<= 1;
            }

            // 9XY0 Skip next if vx == vy
            (9, x, y, 0) => {
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // ANNN Set I to NNN
            (0xa, _, _, _) => {
                let nnn = op & 0x0fff;
                self.i_reg = nnn;
            }

            // BNNN PC = V0 + NNN
            (0xb, _, _, _) => {
                let nnn = op & 0x0fff;
                self.pc = nnn + self.v_reg[0] as u16;
            }

            // CXNN VX = rand & NN
            (0xc, x, _, _) => {
                let nn = (op & 0x00ff) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }

            // DXYN - Draw Sprite
            (0xd, vx, vy, rows) => {
                let x_coord = self.v_reg[vx] as u16;
                let y_coord = self.v_reg[vy] as u16;

                let mut flipped = false;
                for y_line in 0..rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line as u16) as usize % SCREEN_HEIGHT;

                            let idx = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_reg[0xf] = 1;
                } else {
                    self.v_reg[0xf] = 0;
                }
            }

            // EX9E - Skip if key is pressed
            (0xe, x, 9, 0xe) => {
                let vx = self.v_reg[x];
                if self.keys[vx as usize] {
                    self.pc += 2;
                }
            }

            // EXA1 Skip if key is not pressed
            (0xe, x, 0xa, 1) => {
                let vx = self.v_reg[x];
                if !self.keys[vx as usize] {
                    self.pc += 2;
                }
            }

            // FX07 VX = DT
            (0xf, x, 0, 7) => self.v_reg[x] = self.dt,

            // FX0A - Wait for key press
            (0xf, x, 0, 0xa) => {
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            }

            // FX15 - DT = VX
            (0xf, x, 1, 5) => self.dt = self.v_reg[x],

            // FX18 - ST = VX
            (0xf, x, 1, 8) => self.st = self.v_reg[x],

            // FX1E - I += VX
            (0xf, x, 1, 0xe) => self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16),

            // FX29 - Set I to Font address
            (0xf, x, 2, 9) => self.i_reg = self.v_reg[x] as u16 * 5,

            // FX33 - I = BCD(VX)
            (0xf, x, 3, 3) => {
                let vx = self.v_reg[x];
                let hundreds = vx / 100;
                let tens = (vx / 10) % 10;
                let ones = vx % 10;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }

            // FX55 - Store V0 to VX into I
            (0xf, x, 5, 5) => {
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            }

            // FX65 - Load I into V0 - VX
            (0xf, x, 6, 5) => {
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx]
                }
            }

            (_, _, _, _) => unimplemented!(),
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = start + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn fetch(&mut self) -> u16 {
        let higher = self.ram[self.pc as usize] as u16;
        let lower = self.ram[(self.pc + 1) as usize] as u16;

        let opcode = higher << 8 | lower;

        self.pc += 2;
        opcode
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                //sound not implemented
            }
            self.st -= 1;
        }
    }
}
