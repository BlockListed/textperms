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

#![feature(unix_chown)]

mod text;
mod perms;

mod read_perms;
mod write_perms;
mod logging;

use clap::IntoApp;
use clap::Parser;
use std::io::Error;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File to get input from.
    #[clap(short, long)]
    file: String,

    /// Output file
    #[clap(short, long)]
    outfile: Option<String>,

    /// Write permissions to file gotten from input
    #[clap(short, long)]
    write: bool,

    /// Dry-Run (Only applicable to write)
    #[clap(long = "dry-run", short)]
    dryrun: bool,

    /// Force exact path
    #[clap(long = "force-path")]
    forcepath: bool,
}

fn main() -> Result<(), Error> {
    let args = Args::command().before_help(text::LICENSE).before_long_help(text::FULL_LICENSE).get_matches();
    if args.occurrences_of("write") > 0 {
        let dry_run = args.occurrences_of("dryrun") > 0;
        if let Err(x) = write_perms::write_perms(args.value_of("file").unwrap(), dry_run) {
            Args::command().error(
                clap::ErrorKind::InvalidValue, 
                x)
                .exit();
        }
        return Ok(());
    }
    if let Some(x) = args.value_of("outfile") {
        let mut outfile = x.to_string();
        if !outfile.ends_with(".zstd") && args.occurrences_of("forcepath") == 0 && outfile != "-" {
            outfile.push_str(".zstd");
        }
        if let Err(x) = read_perms::read_perms(args.value_of("file").unwrap(), outfile.as_str()) {
                Args::command().error(
                    clap::ErrorKind::InvalidValue, 
                    x)
                    .exit();
        }
    } else {
        Args::command().error(
            clap::ErrorKind::MissingRequiredArgument,
            "--outfile is required when reading permissions!"
        )
        .exit();
    }
    Ok(())
}
