use std::cmp;
use std::fmt;
use rand::random;

use instruction::Instruction;

const PROGRAM_ORIGIN: u32 = 0x200;
const MEMORY_SIZE: usize = 0x1000;
const REGISTER_COUNT: usize = 0x10;
const CALL_STACK_SIZE: usize = 0x20;
const INSTRUCTION_SIZE: u32 = 2; // 16 bit instructions
// const CHARACTER_SPRITE_WIDTH: u32 = 4;
const CHARACTER_SPRITE_HEIGHT: u32 = 5;
const KEYPAD_STATES: usize = 0x10;

pub struct Chip8 {
	memory: [u8; MEMORY_SIZE],
	registers: [u8; REGISTER_COUNT],
	call_stack: Vec<u32>,
	address_register: u32,
	program_counter: u32,
	delay_counter: u32,
	sound_counter: u32,
	cycles: u64,
	key_states: [bool; KEYPAD_STATES],
}

impl fmt::Debug for Chip8 {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "pc: {:x} I: {:x} stack size: {}",
			self.program_counter,
			self.address_register,
			self.call_stack.len()
		));
        try!(writeln!(f, "  V0 {:2x} V1 {:2x} V2 {:2x} V3 {:2x}",
			self.registers[0x0], self.registers[0x1], self.registers[0x2], self.registers[0x3]
		));
        try!(writeln!(f, "  V4 {:2x} V5 {:2x} V6 {:2x} V7 {:2x}",
			self.registers[0x4], self.registers[0x5], self.registers[0x6], self.registers[0x7]
		));
        try!(writeln!(f, "  V8 {:2x} V9 {:2x} VA {:2x} VB {:2x}",
			self.registers[0x8], self.registers[0x9], self.registers[0xA], self.registers[0xB]
		));
        writeln!(f, "  VC {:2x} VD {:2x} VE {:2x} VF {:2x}",
			self.registers[0xC], self.registers[0xD], self.registers[0xE], self.registers[0xF]
		)
    }
}

impl Chip8 {
	pub fn new(rom: Vec<u8>) -> Chip8 {
		let mut initial_ram = [0; MEMORY_SIZE];

		let usable_memory = MEMORY_SIZE - PROGRAM_ORIGIN as usize;
		let start_copy = PROGRAM_ORIGIN as usize;
		let end_copy = cmp::min(usable_memory, rom.len());
		for offset in 0..end_copy {
			initial_ram[start_copy + offset] = rom[offset];
		}
		
		Chip8 {
			memory: initial_ram,
			registers: [0; REGISTER_COUNT],
			call_stack: Vec::with_capacity(CALL_STACK_SIZE),
			address_register: 0,
			program_counter: PROGRAM_ORIGIN,
			delay_counter: 0,
			sound_counter: 0,
			cycles: 0,
			key_states: [false; KEYPAD_STATES],
		}
	}

	pub fn run(&mut self) {
		loop {
			self.step()
		}
	}

	pub fn step(&mut self) {
		let instruction_word = self.read_opcode(self.program_counter);
		let mut advance = INSTRUCTION_SIZE;

		let instruction = Instruction::new(instruction_word);

		if self.cycles % 10000000 == 0 {
			println!("[cycle {} @ {:x}] instruction: {:?}", self.cycles, self.program_counter, instruction);
			// println!("cpu state: {:?}", self);
		}

		match instruction {
			Instruction::CallRCA1802(_) => {
				// apparently rarely implemented?
				panic!("RCA1802 unsupported/unimplemented");
			},
			Instruction::ClearScreen => {
				// TODO: implement
				println!("clear screen unimplemented");
			},
			Instruction::ReturnFromSub => {
				if let Some(addr) = self.call_stack.pop() {
					self.program_counter = addr;
					advance = 0;
				} else {
					panic!("call stack underflow @ {:x}", self.program_counter);
				}
			},
			Instruction::Jump(address) => {
				self.program_counter = address as u32;
				advance = 0;
			},
			Instruction::Call(address) => {
				if self.call_stack.len() >= self.call_stack.capacity() {
					panic!("call stack overflow");
				}
				self.call_stack.push(self.program_counter + INSTRUCTION_SIZE);
				self.program_counter = address as u32;
				advance = 0;
			},
			Instruction::SkipEquals(reg, constant) => {
				if self.read_reg(reg) == constant {
					advance = INSTRUCTION_SIZE * 2;
				}
			},
			Instruction::SkipNotEquals(reg, constant) => {
				if self.read_reg(reg) != constant {
					advance = INSTRUCTION_SIZE * 2;
				}
			},
			Instruction::SkipRegEquals(xreg, yreg) => {
				if self.read_reg(xreg) == self.read_reg(yreg) {
					advance = INSTRUCTION_SIZE * 2;
				}
			},
			Instruction::SetReg(reg, constant) => {
				self.write_reg(reg, constant);
			},
			Instruction::AddConst(reg, constant) => {
				let value = ((self.read_reg(reg) as u16 + constant as u16) & 0xFF) as u8;
				self.write_reg(reg, value);
			},
			Instruction::CopyReg(xreg, yreg) => {
				let value = self.read_reg(yreg);
				self.write_reg(xreg, value);
			},
			Instruction::OrReg(xreg, yreg) => {
				let a = self.read_reg(xreg);
				let b = self.read_reg(yreg);
				self.write_reg(xreg, a | b);
			},
			Instruction::AndReg(xreg, yreg) => {
				let a = self.read_reg(xreg);
				let b = self.read_reg(yreg);
				self.write_reg(xreg, a & b);
			},
			Instruction::XorReg(xreg, yreg) => {
				let a = self.read_reg(xreg);
				let b = self.read_reg(yreg);
				self.write_reg(xreg, a ^ b);
			},
			Instruction::AddReg(xreg, yreg) => {
				let a = self.read_reg(xreg) as u16;
				let b = self.read_reg(yreg) as u16;
				let r = a + b;
				self.write_reg(xreg, (r & 0xFF) as u8);
				match r > u8::max_value() as u16 {
					true => self.write_reg(0xF, 1),
					false => self.write_reg(0xF, 0),
				}
			},
			Instruction::SubReg(xreg, yreg) => {
				let a = self.read_reg(xreg);
				let b = self.read_reg(yreg);
				let r = match a <= b {
					false => {
						self.write_reg(0xF, 0);
						a - b
					},
					true => {
						self.write_reg(0xF, 1);
						u8::max_value() - (b - a)
					},
				};
				self.write_reg(xreg, r);
			},
			Instruction::RightShiftReg(xreg, yreg) => {
				let a = self.read_reg(xreg);
				let b = self.read_reg(yreg);
				let ignore_yreg = true;
				match ignore_yreg {
					true => {
						self.write_reg(0xF, a & 0x1);
						self.write_reg(xreg, a >> 1);
					},
					false => {
						self.write_reg(0xF, b & 0x1);
						self.write_reg(xreg, b >> 1);
					},
				};
			},
			Instruction::SubRegRev(xreg, yreg) => {
				let a = self.read_reg(xreg);
				let b = self.read_reg(yreg);
				let r = match b <= a {
					false => {
						self.write_reg(0xF, 0);
						b - a
					},
					true => {
						self.write_reg(0xF, 1);
						u8::max_value() - (a - b)
					},
				};
				self.write_reg(xreg, r);
			},
			Instruction::LeftShiftReg(xreg, yreg) => {
				let a = self.read_reg(xreg);
				let b = self.read_reg(yreg);
				let ignore_yreg = true;
				match ignore_yreg {
					true => {
						self.write_reg(0xF, a & 0x1);
						self.write_reg(xreg, a << 1);
					},
					false => {
						self.write_reg(0xF, b & 0x1);
						self.write_reg(xreg, b << 1);
					},
				};
			},
			Instruction::SkipRegNotEquals(xreg, yreg) => {
				if self.read_reg(xreg) != self.read_reg(yreg) {
					advance = INSTRUCTION_SIZE * 2;
				}
			},
			Instruction::SetAddressReg(constant) => {
				self.address_register = constant as u32;
			},
			Instruction::JumpIndirect(addr) => {
				self.program_counter = addr as u32 + self.read_reg(0) as u32;
				advance = 0;
			},
			Instruction::RandomNumber(reg, mask) => {
				self.write_reg(reg, random::<u8>() & mask);
			},
			Instruction::DrawSprite(xreg, yreg, lines) => {
				// TODO: implement
				let _ = (xreg, yreg, lines);
				// println!("draw sprite unimplemented");
			},
			Instruction::KeyIsPressed(reg) => {
				if self.is_key_pressed(self.read_reg(reg)) == true {
					advance = INSTRUCTION_SIZE * 2;
				}
			},
			Instruction::KeyIsntPressed(reg) => {
				if self.is_key_pressed(self.read_reg(reg)) == false {
					advance = INSTRUCTION_SIZE * 2;
				}
			},
			Instruction::ReadDelayTimer(reg) => {
				let value = self.delay_counter as u8;
				self.write_reg(reg, value);
			},
			Instruction::AwaitKeyPress(reg) => {
				let mut pressed = None;
				for key_number in 0..KEYPAD_STATES {
					if self.key_states[key_number as usize] {
						pressed = Some(key_number);
						break;
					}
				}
				if let Some(key_number) = pressed {
					self.write_reg(reg, key_number as u8);
				} else {
					advance = 0;
				}
			},
			Instruction::SetDelayTimer(reg) => {
				self.delay_counter = self.read_reg(reg) as u32;
			},
			Instruction::SetSoundTimer(reg) => {
				self.sound_counter = self.read_reg(reg) as u32;
			},
			Instruction::AddAddressReg(reg) => {
				self.address_register += self.read_reg(reg) as u32;
			},
			Instruction::AddressKeySprite(reg) => {
				let value = self.read_reg(reg);
				self.address_register = match (value & 0xF) as u32 {
					char @ 0...9 => ('0' as u32  + (char * CHARACTER_SPRITE_HEIGHT)),
					char @ 0xA...0xF => ('A' as u32 + (char * CHARACTER_SPRITE_HEIGHT)),
					_ => panic!("should be impossible, a value masked with 0xF is outside 0-F range?"),
				};
			},
			Instruction::StoreBCDAtAddress(reg) => {
				let value = self.read_reg(reg);
				let digit1 = ((value / 100) % 10) as u8;
				let digit2 = ((value / 10) % 10) as u8;
				let digit3 = (value % 10) as u8;
				let address = self.address_register;
				self.write_byte(address, digit1);
				self.write_byte(address + 1, digit2);
				self.write_byte(address + 2, digit3);
			},
			Instruction::LoadRegisters(reg_count) => {
				let address = self.address_register as u32;
				for reg_index in 0..reg_count as u32 {
					let value = self.read_byte(address + reg_index);
					self.write_reg(reg_index as u8, value);
				}
			},
			Instruction::Unknown(word) => panic!("unknown instruction: 0x{:x}", word),
			unimplemented => panic!("unimplemented instruction: {:?}", unimplemented),
		};

		let delta_ticks = 1u32; // TODO: actually work out when to decrement counters
		if delta_ticks >= self.delay_counter {
			self.delay_counter = 0;
		} else {
			self.delay_counter -= delta_ticks;
		}
		if delta_ticks >= self.sound_counter {
			self.sound_counter = 0;
		} else {
			self.sound_counter -= delta_ticks;
		}

		self.program_counter += advance;
		self.cycles += 1;
	}

	fn is_key_pressed(&self, key_number: u8) -> bool {
		self.key_states[(key_number & 0xF) as usize]
	}

	fn write_reg(&mut self, reg_number: u8, value: u8) {
		self.registers[(reg_number & 0xF) as usize] = value;
	}

	fn read_reg(&self, reg_number: u8) -> u8 {
		self.registers[(reg_number & 0xF) as usize]
	}

	fn read_opcode(&self, addr: u32) -> u16 {
		match addr as usize {
			addr if addr < MEMORY_SIZE => {
				((self.memory[addr] as u16) << 8) | (self.memory[addr + 1] as u16)
			},
			invalid => panic!("invalid program counter: {:x}", invalid),
		}
	}

	fn read_byte(&self, addr: u32) -> u8 {
		match addr as usize {
			addr if addr < MEMORY_SIZE => {
				self.memory[addr]
			},
			invalid => panic!("invalid memory read (byte): {:x}", invalid),
		}
	}

	fn write_byte(&mut self, addr: u32, value: u8) {
		match addr as usize {
			addr if addr < MEMORY_SIZE => {
				self.memory[addr] = value;
			},
			invalid => panic!("invalid memory write (byte): {:x} (value: {:x})", invalid, value),
		}
	}
}

