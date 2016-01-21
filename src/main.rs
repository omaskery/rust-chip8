
use std::env;
use std::fs::File;
use std::path::Path;
use std::io;
use std::io::Read;
use std::cmp;

const PROGRAM_ORIGIN: u32 = 0x200;
const MEMORY_SIZE: usize = 0x1000;
const REGISTER_COUNT: usize = 0x10;

#[derive(Debug)]
enum Instruction {
	CallRCA1802(u16),			// 0NNN
	ClearScreen,				// 00E0
	ReturnFromSub,				// 00EE
	Jump(u16),					// 1NNN
	Call(u16),					// 2NNN
	SkipEquals(u8, u8),			// 3XNN
	SkipNotEquals(u8, u8),		// 4XNN
	SkipRegEquals(u8, u8), 		// 5XY0
	SetReg(u8, u8), 			// 6XNN
	AddConst(u8, u8),			// 7XNN
	CopyReg(u8, u8),			// 8XY0
	OrReg(u8, u8),				// 8XY1
	AndReg(u8, u8),				// 8XY2
	XorReg(u8, u8),				// 8XY3
	AddReg(u8, u8),				// 8XY4
	SubReg(u8, u8),				// 8XY5
	RightShiftReg(u8),			// 8XY6
	SubRegRev(u8, u8),			// 8XY7
	LeftShiftReg(u8),			// 8XYE
	SkipRegNotEquals(u8, u8),	// 9XY0
	SetAddressReg(u16),			// ANNN
	JumpIndirect(u16),			// BNNN
	RandomNumber(u8, u8),		// CXNN
	DrawSprite(u8, u8, u8),		// DXYN
	KeyIsPressed(u8),			// EX9E
	KeyIsntPressed(u8),			// EXA1
	ReadDelayTimer(u8),			// FX07
	AwaitKeyPress(u8),			// FX0A
	SetDelayTimer(u8),			// FX15
	SetSoundTimer(u8),			// FX18
	AddAddressReg(u8),			// FX1E
	AddressKeySprite(u8),		// FX29
	StoreBCDAtAddress(u8), 		// FX33
	StoreRegisters(u8),			// FX55
	LoadRegisters(u8),			// FX65
	Unknown(u16),				// ????
}

impl Instruction {
	fn new(word: u16) -> Instruction {
		let nibble1 = (word >> 12) as u8;
		let nibble2 = (word >> 8 & 0xF) as u8;
		let nibble3 = (word >> 4 & 0xF) as u8;
		let nibble4 = (word & 0xF) as u8;

		match nibble1 {
			0 => {
				match word & 0xFF {
					0xE0 => Instruction::ClearScreen,
					0xEE => Instruction::ReturnFromSub,
					_ => Instruction::CallRCA1802(word & 0xFFF)
				}
			},
			1 => Instruction::Jump(word & 0xFFF),
			2 => Instruction::Call(word & 0xFFF),
			3 => Instruction::SkipEquals(nibble2, (word & 0xFF) as u8),
			4 => Instruction::SkipNotEquals(nibble2, (word & 0xFF) as u8),
			5 => Instruction::SkipRegEquals(nibble2, nibble3),
			6 => Instruction::SetReg(nibble2, (word & 0xFF) as u8),
			7 => Instruction::AddConst(nibble2, (word & 0xFF) as u8),
			8 => {
				match nibble4 {
					0 => Instruction::CopyReg(nibble2, nibble3),
					1 => Instruction::OrReg(nibble2, nibble3),
					2 => Instruction::AndReg(nibble2, nibble3),
					3 => Instruction::XorReg(nibble2, nibble3),
					4 => Instruction::AddReg(nibble2, nibble3),
					5 => Instruction::SubReg(nibble2, nibble3),
					6 => Instruction::RightShiftReg(nibble2),
					7 => Instruction::SubRegRev(nibble2, nibble3),
					0xE => Instruction::LeftShiftReg(nibble2),
					_ => Instruction::Unknown(word),
				}
			},
			9 => Instruction::SkipRegNotEquals(nibble2, nibble3),
			0xA => Instruction::SetAddressReg(word & 0xFFF),
			0xB => Instruction::JumpIndirect(word & 0xFFF),
			0xC => Instruction::RandomNumber(nibble2, (word & 0xFF) as u8),
			0xD => Instruction::DrawSprite(nibble2, nibble3, nibble4),
			0xE => {
				match word & 0xFF {
					0x9E => Instruction::KeyIsPressed(nibble2),
					0xA1 => Instruction::KeyIsntPressed(nibble2),
					_ => Instruction::Unknown(word),
				}
			},
			0xF => {
				match word & 0xFF {
					0x07 => Instruction::ReadDelayTimer(nibble2),
					0x0A => Instruction::AwaitKeyPress(nibble2),
					0x15 => Instruction::SetDelayTimer(nibble2),
					0x18 => Instruction::SetSoundTimer(nibble2),
					0x1E => Instruction::AddAddressReg(nibble2),
					0x29 => Instruction::AddressKeySprite(nibble2),
					0x33 => Instruction::StoreBCDAtAddress(nibble2),
					0x55 => Instruction::StoreRegisters(nibble2),
					0x65 => Instruction::LoadRegisters(nibble2),
					_ => Instruction::Unknown(word),
				}
			}
			_ => Instruction::Unknown(word),
		}
	}
}

struct Chip8 {
	memory: [u8; MEMORY_SIZE],
	registers: [u8; REGISTER_COUNT],
	address_register: u32,
	program_counter: u32,
}

impl Chip8 {
	fn new(rom: Vec<u8>) -> Chip8 {
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
			address_register: 0,
			program_counter: PROGRAM_ORIGIN,
		}
	}

	fn run(&mut self) {
		loop {
			self.step()
		}
	}

	fn step(&mut self) {
		let instruction_word = self.read_opcode(self.program_counter);
		let advance = 2;

		let instruction = Instruction::new(instruction_word);

		println!("instruction: {:?}", instruction);

		match instruction {
			Instruction::SetReg(reg, constant) => {
				self.registers[(reg & 0xF) as usize] = constant
			},
			Instruction::DrawSprite(xreg, yreg, lines) => {
				// TODO: implement
			},
			Instruction::SetAddressReg(reg) => {
				self.address_register = self.registers[(reg & 0xF) as usize] as u32
			},
			Instruction::Unknown(word) => panic!("unknown instruction: 0x{:x}", word),
			unimplemented => panic!("unimplemented instruction: {:?}", unimplemented),
		};

		self.program_counter += advance;
	}

	fn read_opcode(&self, addr: u32) -> u16 {
		match addr as usize {
			addr if addr < MEMORY_SIZE => {
				((self.memory[addr] as u16) << 8) | (self.memory[addr + 1] as u16)
			},
			invalid => panic!("invalid program counter: {}", invalid),
		}
	}
}

#[derive(Debug)]
enum EmulatorError {
	BadArgs(String),
	IoError(io::Error),
}

impl From<io::Error> for EmulatorError {
	fn from(err: io::Error) -> EmulatorError {
		EmulatorError::IoError(err)
	}
}

fn main() {
	let rom_filepath = env::args().nth(1);
	
	if let Err(e) = run_emulator(rom_filepath) {
		println!("error: {:?}", e);
	}
}

fn run_emulator(path: Option<String>) -> Result<(), EmulatorError> {
	if let Some(path) = path {
		let filepath = Path::new(&path);

		let pretty_name = match filepath.file_name() {
			Some(filename) => filename.to_string_lossy().into_owned(),
			_ => filepath.to_string_lossy().into_owned(),
		};
		print!("loading rom '{}'... ", pretty_name);
		let rom = try!(load_file(filepath));
		println!("done.");

		let mut chip8 = Chip8::new(rom);
		chip8.run();

		Ok(())
	} else {
		Err(EmulatorError::BadArgs("expected a filepath to a ROM to execute".into()))
	}
}

fn load_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
	let mut file = try!(File::open(path));
	let mut buffer = Vec::new();

	try!(file.read_to_end(&mut buffer));

	Ok(buffer)
}

