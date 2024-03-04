use std::process::ExitCode;

use clap::Parser;
use nibarchive::NIBArchive;
use nibarchive_upgrade::upgrade;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to an input NIB Archive .nib file
    nib_archive_path: String,

    /// Path to an output .plist file
    plist_path: String,
}

pub fn main() -> ExitCode {
    let args = Args::parse();
    let archive = NIBArchive::from_file(args.nib_archive_path);
    if let Err(e) = archive {
        println!("{e}");
        return ExitCode::FAILURE;
    }
    let plist_value = upgrade(&archive.unwrap());
    if let Err(e) = plist_value.to_file_binary(args.plist_path) {
        println!("{e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
