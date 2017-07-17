#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate url;

mod error {
    use url::ParseError;

    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            UrlParse(ParseError);
        }
    }
}

use clap::{Arg, App, AppSettings, SubCommand};
use error::*;
use url::Url;
use url::form_urlencoded::byte_serialize;

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

    let out = match command.action {
        Action::Encode => {
            if command.verbose {
                println!("Encoding");
            }
            encode(&command.input)?
        },
        //Action::Decode => {
        //    if verbose {
        //        println!("Decoding");
        //    }
        //    decode(&command.input)?
        //},
        _ => "".to_owned(),
    };

    println!("{}", out);

    Ok(())
}

fn encode(s: &str) -> Result<String> {
    Url::parse(s).map(|url| {
        let query = url.query_pairs()
            .map(|(k,v)| {
                format!("{}={}",
                    byte_serialize(k.as_bytes()).collect::<String>(),
                    byte_serialize(v.as_bytes()).collect::<String>(),
                )
            })
            .collect::<Vec<_>>() // use itertools intersperse?
            .join("&");

        let mut out = format!("{}://", url.scheme());
        if let Some(host) = url.host_str() {
            out.push_str(host);
        }
        out.push_str(&format!("{}?{}", url.path(), query));

        if let Some(frag) = url.fragment() {
            out.push_str("#");
            out.push_str(frag);
        }

        out
    })
    .chain_err(|| "No Url found to encode")
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encoding() {
        let input = "https://github.com/hw chen/aggregate?one=[two =three].[four]&five=six";
        let correct_out = "https://github.com/hw%20chen/aggregate?one=%5Btwo+%3Dthree%5D.%5Bfour%5D&five=six";
        assert_eq!(encode(input).unwrap(), correct_out.to_owned());

    }

    #[test]
    #[should_panic]
    fn test_encoding_bad_input() {
        let input = "hw chen/aggregate?one=[two =three].[four]&five=six";
        encode(input).unwrap();
    }
}
