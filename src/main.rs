extern crate rand;

mod instruction;
mod chip8;

use std::env;
use std::fs::File;
use std::path::Path;
use std::io;
use std::io::Read;

use chip8::Chip8;

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

