
#[derive(Debug)]
pub enum Instruction {
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
	RightShiftReg(u8, u8),		// 8XY6
	SubRegRev(u8, u8),			// 8XY7
	LeftShiftReg(u8, u8),		// 8XYE
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
	pub fn new(word: u16) -> Instruction {
		let nibble1 = (word >> 12) as u8;
		let nibble2 = (word >> 8 & 0xF) as u8;
		let nibble3 = (word >> 4 & 0xF) as u8;
		let nibble4 = (word & 0xF) as u8;

		match nibble1 {
			0x0 => match word & 0xFF {
				0xE0 => Instruction::ClearScreen,
				0xEE => Instruction::ReturnFromSub,
				_ => Instruction::CallRCA1802(word & 0xFFF)
			},
			0x1 => Instruction::Jump(word & 0xFFF),
			0x2 => Instruction::Call(word & 0xFFF),
			0x3 => Instruction::SkipEquals(nibble2, (word & 0xFF) as u8),
			0x4 => Instruction::SkipNotEquals(nibble2, (word & 0xFF) as u8),
			0x5 => Instruction::SkipRegEquals(nibble2, nibble3),
			0x6 => Instruction::SetReg(nibble2, (word & 0xFF) as u8),
			0x7 => Instruction::AddConst(nibble2, (word & 0xFF) as u8),
			0x8 => match nibble4 {
				0x0 => Instruction::CopyReg(nibble2, nibble3),
				0x1 => Instruction::OrReg(nibble2, nibble3),
				0x2 => Instruction::AndReg(nibble2, nibble3),
				0x3 => Instruction::XorReg(nibble2, nibble3),
				0x4 => Instruction::AddReg(nibble2, nibble3),
				0x5 => Instruction::SubReg(nibble2, nibble3),
				0x6 => Instruction::RightShiftReg(nibble2, nibble3),
				0x7 => Instruction::SubRegRev(nibble2, nibble3),
				0xE => Instruction::LeftShiftReg(nibble2, nibble3),
				_ => Instruction::Unknown(word),
			},
			0x9 => Instruction::SkipRegNotEquals(nibble2, nibble3),
			0xA => Instruction::SetAddressReg(word & 0xFFF),
			0xB => Instruction::JumpIndirect(word & 0xFFF),
			0xC => Instruction::RandomNumber(nibble2, (word & 0xFF) as u8),
			0xD => Instruction::DrawSprite(nibble2, nibble3, nibble4),
			0xE => match word & 0xFF {
				0x9E => Instruction::KeyIsPressed(nibble2),
				0xA1 => Instruction::KeyIsntPressed(nibble2),
				_ => Instruction::Unknown(word),
			},
			0xF => match word & 0xFF {
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
			},
			_ => Instruction::Unknown(word),
		}
	}
}

