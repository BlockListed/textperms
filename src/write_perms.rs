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

use std::fs::File;
use std::fs::metadata;
use std::os::unix::fs::chown;
use std::fs::set_permissions;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::io::BufReader;
use std::io::Read;

use crossbeam::channel;
use crossbeam::scope;
use quick_protobuf::MessageRead;
use zstd::Decoder;
use quick_protobuf::BytesReader;

use crate::perms;
use crate::logging;

type StrResult = Result<(), String>;

type PermMap<'a> = (String, perms::permission);
type PermSender<'a> = crossbeam::channel::Sender<PermMap<'a>>;
type PermReceiver<'a> = crossbeam::channel::Receiver<PermMap<'a>>;
type ErrSender = crossbeam::channel::Sender<StrResult>;
type ErrReceiver = crossbeam::channel::Receiver<StrResult>;

pub fn write_perms(input: &str, dry_mode: bool) -> StrResult {
	let (tx, rx): (PermSender, PermReceiver) = channel::unbounded();
	let (etx, erx): (ErrSender, ErrReceiver) = channel::unbounded();

	let avail_cpus = std::cmp::max(num_cpus::get() - 1, 1);

	scope(|s| {
		s.spawn(|_| {
			let errtx = etx.clone();
			parse_input(input, tx, errtx);
		});

		for _ in 0..avail_cpus {
			s.spawn(|_| {
				let r = rx.clone();
				let e = etx.clone();
				write_values(r, e, dry_mode);
			});
		}
	}).unwrap();

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

fn parse_input(input: &str, tx: PermSender, etx: ErrSender) {
	let mut infile = match Decoder::new(BufReader::new(match File::open(input) {
		Ok(x) => x,
		Err(x) => {
			etx.send(Err(logging::format_log(file!(), line!(), &x.to_string()))).unwrap();
			return;
		}
	})) {
		Ok(x) => x,
		Err(x) => {
			etx.send(Err(logging::format_log(file!(), line!(), &x.to_string()))).unwrap();
			return;
		}
	};

	let size = metadata(input).unwrap().len();

	let mut bytes: Vec<u8> = Vec::with_capacity(size.try_into().unwrap());
	match infile.read_to_end(&mut bytes) {
		Ok(_) => (),
		Err(x) => {
			etx.send(Err(logging::format_log(file!(), line!(), &x.to_string()))).unwrap();
			return;
		}
	};
	let mut bytesreader = BytesReader::from_bytes(&bytes);
	let permissions = match perms::permissions::from_reader(&mut bytesreader, &bytes) {
		Ok(x) => x,
		Err(x) => {
			etx.send(Err(logging::format_log(file!(), line!(), &x.to_string()))).unwrap();
			return;
		}
	};
	for (k, v) in permissions.permission {
		if tx.send((k.to_string(), v)).is_err() {};
	}
}

fn write_values(rx: PermReceiver, etx: ErrSender, dry_mode: bool) {
	loop {
		let file_data = match rx.try_recv() {
			Err(channel::TryRecvError::Disconnected) => {
				break;
			},
			Err(channel::TryRecvError::Empty) => {
				continue;
			},
			Ok(x) => x
		};
		if !dry_mode {
			match chown(&file_data.0, Some(file_data.1.uid), Some(file_data.1.gid)) {
				Ok(_) => (),
				Err(x) => {
					etx.send(Err(logging::format_log(file!(), line!(), &x.to_string()))).unwrap();
					continue;
				}
			};
			let permissions = Permissions::from_mode(file_data.1.mode);
			match set_permissions(&file_data.0, permissions) {
				Ok(_) => (),
				Err(x) => {
					etx.send(Err(logging::format_log(file!(), line!(), &x.to_string()))).unwrap();
					continue;
				}
			}
		} else {
			let s = format!("{:#o}", file_data.1.mode);

			let mut mode = s.chars().rev().take(3).collect::<String>();
			mode = mode.chars().rev().collect::<String>();
			println!("chown {1}:{2} \"{0}\"; chmod {3} \"{0}\"", file_data.0, file_data.1.uid, file_data.1.gid, mode)
		}
	}
}