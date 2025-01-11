const MEMORY_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const FONTSET_SIZE: usize = 80;
const FONTSET_START_ADDRESS: usize = 0x50;
const PROGRAM_START_ADDRESS: usize = 0x200;

type OpcodeHandler = fn(&mut Chip8, u16);

pub struct Chip8 {
    pub memory: [u8; MEMORY_SIZE],       // 4kb memory
    pub registers: [u8; REGISTER_COUNT], // 16 general purpose registers
    pub index_register: u16,
    pub program_counter: u16,
    pub screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT], // 64x32 pixel display
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack: [u16; STACK_SIZE], // stack with 16 levels
    pub stack_pointer: u8,        // stack pointer
    pub keys: [u8; REGISTER_COUNT],
    pub jump_table: [OpcodeHandler; 16],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut chip8 = Chip8 {
            memory: [0; MEMORY_SIZE], //figure it retard
            registers: [0; REGISTER_COUNT],
            index_register: 0,
            program_counter: PROGRAM_START_ADDRESS as u16, // Programs start at 0x200
            screen: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            keys: [0; REGISTER_COUNT],
            jump_table: Chip8::create_jump_table(),
        };
        chip8.load_fonts();
        chip8
    }

    pub fn load_fonts(&mut self) {
        let fontset: [u8; FONTSET_SIZE] = [
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
        for (i, &byte) in fontset.iter().enumerate() {
            self.memory[FONTSET_START_ADDRESS + i] = byte;
        }
    }

    fn create_jump_table() -> [OpcodeHandler; 16] {
        [
            Chip8::op_0xxx,          // 0x0XXX group
            Chip8::jp,               // 0x1XXX
            Chip8::call,             // 0x2XXX
            Chip8::se_vx_byte,       // 0x3XXX
            Chip8::sne_vx_byte,      // 0x4XXX
            Chip8::se_vx_vy,         // 0x5XXX
            Chip8::ld_vx_byte,       // 0x6XXX
            Chip8::add_vx_byte,      // 0x7XXX
            Chip8::op_8xxx,          // 0x8XXX group
            Chip8::sne_vx_vy,        // 0x9XXX
            Chip8::ld_i_addr,        // 0xAXXX
            Chip8::jp_v0_addr,       // 0xBXXX
            Chip8::rnd_vx_byte,      // 0xCXXX
            Chip8::drw_vx_vy_nibble, // 0xDXXX
            Chip8::op_exxx,          // 0xEXXX group
            Chip8::op_fxxx,          // 0xFXXX group
        ]
    }

    pub fn load_program(&mut self, program: &[u8]) {
        for (i, &byte) in program.iter().enumerate() {
            self.memory[0x200 + i] = byte;
        }
    }

    pub fn emulate_cycle(&mut self) {
        let opcode = self.fetch_opcode();
        self.program_counter += 2;
        let index = (opcode & 0xF000) >> 12;
        let handler = self.jump_table[index as usize];
        handler(self, opcode);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn fetch_opcode(&self) -> u16 {
        let high_byte = self.memory[self.program_counter as usize] as u16;
        let low_byte = self.memory[(self.program_counter + 1) as usize] as u16;
        (high_byte << 8) | low_byte
    }

    fn op_0xxx(&mut self, opcode: u16) {
        match opcode & 0x00FF {
            0x00E0 => self.cls(),
            0x00EE => self.ret(),
            _ => unimplemented!("opcode {:04x} not implemented", opcode),
        }
    }

    fn op_8xxx(&mut self, opcode: u16) {
        match opcode & 0x000F {
            0x0000 => self.ld_vx_vy(opcode),
            0x0001 => self.or_vx_vy(opcode),
            0x0002 => self.and_vx_vy(opcode),
            0x0003 => self.xor_vx_vy(opcode),
            0x0004 => self.add_vx_vy(opcode),
            0x0005 => self.sub_vx_vy(opcode),
            0x0006 => self.shr_vx(opcode),
            0x0007 => self.subn_vx_vy(opcode),
            0x000E => self.shl_vx(opcode),
            _ => unimplemented!("Opcode {:04x} not implemented", opcode),
        }
    }

    fn op_exxx(&mut self, opcode: u16) {
        match opcode & 0x00FF {
            0x009E => self.skp_vx(opcode),
            0x00A1 => self.sknp_vx(opcode),
            _ => unimplemented!("Opcode {:04x} not implemented", opcode),
        }
    }

    fn op_fxxx(&mut self, opcode: u16) {
        match opcode & 0x00FF {
            0x0007 => self.ld_vx_dt(opcode),
            0x000A => self.ld_vx_k(opcode),
            0x0015 => self.ld_dt_vx(opcode),
            0x0018 => self.ld_st_vx(opcode),
            0x001E => self.add_i_vx(opcode),
            0x0029 => self.ld_f_vx(opcode),
            0x0033 => self.ld_b_vx(opcode),
            0x0055 => self.ld_i_vx(opcode),
            0x0065 => self.ld_vx_i(opcode),
            _ => unimplemented!("Opcode {:04x} not implemented", opcode),
        }
    }

    // instruction implementation===============================================
    // CLS - 00E0
    // Instruction: clear the display
    fn cls(&mut self) {
        self.screen = [0; SCREEN_WIDTH * SCREEN_HEIGHT]
    }

    // RET - 00EE
    // Instruction: return from a subroutine
    fn ret(&mut self) {
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
    }

    // JP - 1NNN
    // Instruction: jump to address NNN
    fn jp(&mut self, opcode: u16) {
        let address = opcode & 0x0FFF;
        self.program_counter = address;
    }

    // CALL - 2NNN
    // Instruction: call subroutine at NNN
    fn call(&mut self, opcode: u16) {
        let address = opcode & 0x0FFF;
        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = address;
    }

    // SE Vx, byte - 3XNN
    // Instruction: skip next instruction if Vx equals NN
    fn se_vx_byte(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let byte = (opcode & 0x00FF) as u8;
        if self.registers[x] == byte {
            self.program_counter += 2;
        }
    }

    // SNE Vx, byte - 4XNN
    // Instruction: skip next instruction if Vx doesn't equal NN
    fn sne_vx_byte(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let byte = (opcode & 0x00FF) as u8;
        if self.registers[x] != byte {
            self.program_counter += 2;
        }
    }

    // SE Vx, Vy - 5XY0
    // Instruction: skip next instruction if Vx equals Vy
    fn se_vx_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.registers[x] == self.registers[y] {
            self.program_counter += 2;
        }
    }

    // LD Vx, byte - 6XNN
    // Instruction: set Vx to NN
    fn ld_vx_byte(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let byte = (opcode & 0x00FF) as u8;
        self.registers[x] = byte;
    }

    // ADD Vx, byte - 7XNN
    // Instruction: add NN to Vx
    fn add_vx_byte(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let byte = (opcode & 0x00FF) as u8;
        self.registers[x] = self.registers[x].wrapping_add(byte);
    }

    // LD Vx, Vy - 8XY0
    // Instruction: set Vx to the value of Vy
    fn ld_vx_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.registers[x] = self.registers[y];
    }

    // OR Vx, Vy - 8XY1
    // Instruction: set Vx to Vx OR Vy
    fn or_vx_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.registers[x] |= self.registers[y];
    }

    // AND Vx, Vy - 8XY2
    // Instruction: set Vx to Vx AND Vy
    fn and_vx_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.registers[x] &= self.registers[y];
    }

    // XOR Vx, Vy - 8XY3
    // Instruction: set Vx to Vx XOR Vy
    fn xor_vx_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.registers[x] ^= self.registers[y];
    }

    // ADD Vx, Vy - 8XY4
    // Instruction: Add Vy to Vx, set VF = carry
    fn add_vx_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (result, carry) = self.registers[x].overflowing_add(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = if carry { 1 } else { 0 };
    }

    // SUB Vx, Vy - 8XY5
    // Instruction: subtract Vy from Vx, set VF = NOT borrow
    fn sub_vx_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (result, borrow) = self.registers[x].overflowing_sub(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = if borrow { 0 } else { 1 };
    }

    // SHR Vx - 8XY6
    // Instruction: set Vx = Vx SHR 1
    fn shr_vx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.registers[0xF] = self.registers[x] & 0x1;
        self.registers[x] >>= 1;
    }

    // SUBN Vx, Vy - 8XY7
    // Instruction: set Vx = Vy - Vx, set VF = NOT borrow
    fn subn_vx_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.registers[0xF] = if self.registers[y] > self.registers[x] {
            1
        } else {
            0
        };
        self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
    }

    // SHL Vx - 8XYE
    // Instruction: set Vx = Vx SHL 1
    fn shl_vx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.registers[0xF] = (self.registers[x] & 0x80) >> 7;
        self.registers[x] <<= 1;
    }

    // SNE Vx, Vy - 9XY0
    // Instruction: skip the next instruction if Vx != Vy
    fn sne_vx_vy(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.registers[x] != self.registers[y] {
            self.program_counter += 2;
        }
    }

    // LD I, addr - ANNN
    // Instruction: set I = NNN
    fn ld_i_addr(&mut self, opcode: u16) {
        let address = opcode & 0x0FFF;
        self.index_register = address;
    }

    // JP V0, addr - BNNN
    // Instruction: jump to location nnn + V0
    fn jp_v0_addr(&mut self, opcode: u16) {
        let address = opcode & 0x0FFF;
        self.program_counter = (self.registers[0] as u16) + address;
    }

    // RND Vx, byte
    // Instruction: set Vx = random byte and passed in byte
    // random byte is just 0x01 for now
    fn rnd_vx_byte(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let byte = (opcode & 0x00FF) as u8;
        let random_byte: u8 = 0x01; // Generate a random byte(just 0x01 for now ok)
        self.registers[x] = random_byte & byte;
    }

    // DRW Vx, Vy, nibble
    // Instruction: display n-byte sprite starting at memory location I at (Vx, Vy), set VF =
    // collision
    fn drw_vx_vy_nibble(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let height = (opcode & 0x000F) as usize;

        let vx = self.registers[x] as usize;
        let vy = self.registers[y] as usize;

        self.registers[0xF] = 0;

        for row in 0..height {
            let sprite_byte = self.memory[self.index_register as usize + row];
            for col in 0..8 {
                let sprite_pixel = sprite_byte & (0x80 >> col);
                let screen_index = (vy + row) * SCREEN_WIDTH + (vx + col);

                if screen_index < self.screen.len() {
                    let screen_pixel = &mut self.screen[screen_index];
                    if sprite_pixel != 0 {
                        if *screen_pixel == 1 {
                            self.registers[0xF] = 1;
                        }
                        *screen_pixel ^= 1;
                    }
                }
            }
        }
    }

    // SKP Vx - EX9E
    // Instruction: skip the next instruction if the key with the value of Vx is pressed
    fn skp_vx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let key = self.registers[x];
        if self.keys[key as usize] != 0 {
            self.program_counter += 2;
        }
    }

    // SKNP Vx - EXA1
    // Instruction: skip the next instruction if the key with the value of Vx is not pressed
    fn sknp_vx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let key = self.registers[x];
        if self.keys[key as usize] == 0 {
            self.program_counter += 2;
        }
    }

    // LD Vx, DT - FX07
    // Instruction: set Vx = delay timer value
    fn ld_vx_dt(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.registers[x] = self.delay_timer;
    }

    // LD Vx, K - FX0A
    // Instruction: wait for a key press, store the value of the key in Vx
    fn ld_vx_k(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        for i in 0..self.keys.len() {
            if self.keys[i] != 0 {
                self.registers[x] = i as u8;
                return;
            }
        }
        // If no key is pressed, decrement PC to repeat the instruction
        self.program_counter -= 2;
    }

    // LD DT, Vx - FX15
    // Instruction: set delay timer = Vx
    fn ld_dt_vx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.delay_timer = self.registers[x];
    }

    // LD ST, Vx - FX18
    // Instruction: set sound timer = Vx
    fn ld_st_vx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.sound_timer = self.registers[x];
    }

    // ADD I, Vx - FX1E
    // Instruction: Set I = I + Vx
    fn add_i_vx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.index_register = self.index_register.wrapping_add(self.registers[x] as u16);
    }

    // LD F, Vx - FX29
    // Instruction: set I = location of sprite for digit Vx
    fn ld_f_vx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let digit = self.registers[x] as u16;
        self.index_register = FONTSET_START_ADDRESS as u16 + digit * 5;
    }

    // LD B, Vx
    // Instruction: store BCD representation of Vx in memory locations I, I+1, and I+2
    fn ld_b_vx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let value = self.registers[x];

        self.memory[self.index_register as usize] = value / 100;
        self.memory[(self.index_register + 1) as usize] = (value / 10) % 10;
        self.memory[(self.index_register + 2) as usize] = value % 10;
    }

    // LD [I], Vx
    // Instruction: store registers V0 through Vx in memory starting at location I
    fn ld_i_vx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        for i in 0..=x {
            self.memory[self.index_register as usize + i] = self.registers[i];
        }
    }

    // LD Vx, I
    // Instruction: read registers V0 through Vx from memory starting at location I
    fn ld_vx_i(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        for i in 0..=x {
            self.registers[i] = self.memory[self.index_register as usize + i];
        }
    }
}
