#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate percent_encoding;

mod error {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
        }
    }
}

use clap::{Arg, App, AppSettings, SubCommand};
use error::*;
use std::process;

fn main() {
    if let Err(ref err) = run() {
        println!("error: {}", err);

        for e in err.iter().skip(1) {
            println!(" cause by: {}", e);
        }

        if let Some(backtrace) = err.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {

    // get cli command
    let command = cli_command()
        .chain_err(|| "Error getting command")?;
    Ok(())
}

fn cli_command() -> Result<CommandConfig> {
    let app_m = App::new("Percent Encoding Cli")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("verbose")
             .short("v")
             .global(true))
        .subcommand(SubCommand::with_name("encode")
            .about("encode string with percent encoding")
            .alias("e")
            .arg(Arg::with_name("encode_input")
                .takes_value(true)
                .help("enter string to encode")))
        .subcommand(SubCommand::with_name("decode")
            .about("decode string with percent encoding")
            .alias("d")
            .arg(Arg::with_name("decode_input")
                .takes_value(true)
                .help("enter string to decode")))
        .get_matches();

    // for global flags. Check at each level/subcommand if the flag is present,
    // then flip switch.
    let mut verbose = app_m.is_present("verbose");

    // Now section on matching subcommands and flags
    match app_m.subcommand() {
        ("encode", Some(sub_m)) => {
            if sub_m.is_present("verbose") { verbose = true; }

            let input = sub_m
                .value_of("encode_input")
                .ok_or("Input string required for encoding")?;

            Ok(CommandConfig{
                action: Action::Encode,
                input: input.to_owned(),
                verbose: verbose,
            })
        },
        ("decode", Some(sub_m)) => {
            if sub_m.is_present("verbose") { verbose = true; }

            let input = sub_m
                .value_of("decode_input")
                .ok_or("Input string required for decoding")?;

            Ok(CommandConfig{
                action: Action::Decode,
                input: input.to_owned(),
                verbose: verbose,
            })
        },
        _ => Err("Not a valid subcommand".into()),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandConfig {
    pub action: Action,
    pub input: String, // change to &str later
    pub verbose: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Encode,
    Decode,
}
