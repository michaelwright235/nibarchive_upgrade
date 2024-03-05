use std::process::ExitCode;

use clap::Parser;
use nibarchive::NIBArchive;
use nibarchive_upgrade::upgrade;

/// Convert Apple's NIB Archive .nib to Cocoa Keyed Archive (NSKeyedArchive) .plist.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to an input NIB Archive .nib file
    nib_in: String,

    /// Path to an output .plist file
    plist_out: String,
}

pub fn main() -> ExitCode {
    let args = Args::parse();
    let archive = NIBArchive::from_file(args.nib_in);
    if let Err(e) = archive {
        println!("{e}");
        return ExitCode::FAILURE;
    }
    let plist_value = upgrade(&archive.unwrap());
    if let Err(e) = plist_value.to_file_binary(args.plist_out) {
        println!("{e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
