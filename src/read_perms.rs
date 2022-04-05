// textperms: save uids, gids, and modes of unix files to a textfile and apply them
// Copyright (C) 2022  BlockListed
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::fs::OpenOptions;
use std::io::{BufWriter, Read};

use crate::perms::{self, permission};
use crate::logging;

use crossbeam::channel::unbounded;
use crossbeam::channel;
use crossbeam::scope;

use quick_protobuf::MessageWrite;

use zstd::Encoder;

type ProtoString<'a> = Cow<'a, str>;
type SendChannel<'a> = (crossbeam::channel::Sender<ProtoString<'a>>, crossbeam::channel::Receiver<ProtoString<'a>>);
type StrResult = Result<(), String>;
type ProtoHashMap<'a> = HashMap<ProtoString<'a>, permission>;

pub fn read_perms(input: &str, outfile: &str) -> StrResult {
	let (tx, rx): SendChannel = unbounded();
	let (etx, erx): (crossbeam::channel::Sender<StrResult>, crossbeam::channel::Receiver<StrResult>) = unbounded();
	let mut output: ProtoHashMap  = HashMap::new();
	let output_shared: Arc<Mutex<&mut ProtoHashMap>> = Arc::new(Mutex::new(&mut output));

	let avail_cpus = std::cmp::max(num_cpus::get() - 1, 1);

	scope(|s| {
		s.spawn(|_| {
			parse_input(input, tx, etx);
		});
		for _ in 0..avail_cpus {
			let o = output_shared.clone();
			let r = rx.clone();
			s.spawn(|_| {
				lookup_values(o, r)
			});
			
		}
	}).unwrap();

	match outfile {
		"-" => {
			write_values_stdout(output)?;
		},
		x => {
			write_values(output, x)?;
		}
	}

	let mut err = "".to_string();

	while !erx.is_empty() {
		if let Err(x) = erx.recv().unwrap() {
			err.push_str(x.as_str());
			err.push('\n')
		}
	}

	if !err.is_empty() {
		return Err(err);
	}

	Ok(())
}

fn parse_input(input: &str, tx: crossbeam::channel::Sender<ProtoString>, etx: crossbeam::channel::Sender<StrResult>) {
	let mut instr: String = String::new();

	if input == "-" {
		if let Err(x) = std::io::stdin().read_to_string(&mut instr) {
			etx.send(Err(logging::format_log(file!(), line!(), &x.to_string()))).unwrap();
			return;
		}
	} else {
		instr = match fs::read_to_string(input) {
			Ok(x) => x,
			Err(x) => {
				etx.send(Err(logging::format_log(file!(), line!(), &x.to_string()))).unwrap();
				return;
			}
		};
	}
	for i in instr.split('\n') {
		match tx.send(i.to_string().try_into().unwrap()) {
			Ok(_) => (),
			Err(x) => {
				etx.send(Err(logging::format_log(file!(), line!(), &x.to_string()))).unwrap();
				
				return;
			}
		};
	}
}

fn lookup_values<'a>(output: Arc<Mutex<&mut ProtoHashMap<'a>>>, rx: crossbeam::channel::Receiver<Cow<'a, str>>) {
	let mut cache_vec: Vec<(Cow<'_, str>, perms::permission)> = Vec::with_capacity(50);
	loop {
		let file_path = match rx.try_recv() {
			Err(channel::TryRecvError::Disconnected) => {
				break;
			},
			Err(channel::TryRecvError::Empty) => {
				continue;
			},
			Ok(x) => x
		};
		let file_metadata = match fs::metadata(file_path.to_string()) {
			Err(_) => continue,
			Ok(x) => x,
		};
		let perm = perms::permission {
			mode: file_metadata.mode(),
			uid: file_metadata.uid(),
			gid: file_metadata.gid()
		};
		if let Ok(mut x) = output.try_lock() {
			x.insert(file_path, perm);
			while !cache_vec.is_empty() {
				let p = cache_vec.pop().unwrap();
				x.insert(p.0, p.1);
			}
		} else {
			cache_vec.append(&mut vec![(file_path, perm)]);
		}
	}
}

fn write_values_stdout(input: ProtoHashMap) -> StrResult {
	let o = perms::permissions {
		permission: input
	};

	let mut outfile = match Encoder::new(BufWriter::new(std::io::stdout()), 3) {
		Ok(x) => x,
		Err(x) => {
			return Err(logging::format_log(file!(), line!(), &x.to_string()));
		}
	};

	match outfile.multithread((num_cpus::get()-1).try_into().unwrap()) {
		Ok(_) => (),
		Err(x) => {
			return Err(logging::format_log(file!(), line!(), &x.to_string()));
		}
	};

	if o.write_message(&mut quick_protobuf::Writer::new(&mut outfile)).is_err() {
		return Err(logging::format_log(file!(), line!(), "Couldn't writer error to output file"));
	};
	outfile.finish().unwrap();
	Ok(())
}

fn write_values(input: ProtoHashMap, output: &str) -> StrResult {
	let o = perms::permissions {
		permission: input
	};
	let p = std::path::Path::new(output);
	if p.is_file() {
		match std::fs::remove_file(p) {
			Ok(_) => (),
			Err(x) => {
				return Err(logging::format_log(file!(), line!(), &x.to_string()));
			}
		};
	} else if p.is_dir() {
		return Err(logging::format_log(file!(), line!(), &format!("{} is a directory!", output)));
	}
	let mut outfile = match Encoder::new(
		BufWriter::new(
			match OpenOptions::new().create(true).write(true).open(output) {
		Ok(x) => x,
		Err(x) => {
			return Err(logging::format_log(file!(), line!(), &x.to_string()));
		}
	}), 3) {
		Ok(x) => x,
		Err(x) => {
			return Err(logging::format_log(file!(), line!(), &x.to_string()));
		}
	};
	match outfile.multithread((num_cpus::get()-1).try_into().unwrap()) {
		Ok(_) => (),
		Err(x) => {
			return Err(logging::format_log(file!(), line!(), &x.to_string()));
		}
	};
	if o.write_message(&mut quick_protobuf::Writer::new(&mut outfile)).is_err() {
		return Err(logging::format_log(file!(), line!(), "Couldn't writer error to output file"));
	};
	outfile.finish().unwrap();

	Ok(())
}