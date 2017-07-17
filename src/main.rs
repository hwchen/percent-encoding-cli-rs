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
use std::collections::HashMap;

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

fn query_pairs(url: &Url) -> HashMap<String, String> {
    enum target {
        Ampersand,
        EqualSign,
    }
    let mut res = HashMap::new();

    // first get just queries
    let url = url.to_string();
    if let Some(q_index) = url.find("?") {
        let queries = url.split(q_index);

        // create index (in reverse) of = and & delimiters
        let mut cut_indices = Vec::new();
        let mut query_sub = queries;
        let mut needle = target::EqualSign;

        while let Some(i) = query_sub.rfind() {
            cut_indices.push(i);
        }

        for i in cut_indices.iter().rev().chunk( {

        }
    }
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
    use url::form_urlencoded::byte_serialize;
    use super::*;

    #[test]
    fn test_encoding() {
        let input = "https://github.com/hw chen/aggregate?one=[two =three].[four]&five=six";
        let correct_out = "https://github.com/hw%20chen/aggregate?one=%5Btwo+%3Dthree%5D.%5Bfour%5D&five=six";
        assert_eq!(encode(input).unwrap(), correct_out.to_owned());

        let input = "http://cny-aardvark.datawheel.us:5000/cubes/acs_ygl_language_proficiency_detailed_5/aggregate?drilldown[]=[Year].[Year]&drilldown[]=[English Proficiency].[English Proficiency]&drilldown[]=[Native Language].[Native Language]&cut[]=[Additive Geography].[Tract].&[14000US36053030501]&measures[]=Population Sum&nonempty=true&distinct=false&parents=false&debug=false";
        let correct_out = "http://cny-aardvark.datawheel.us/cubes/acs_ygl_language_proficiency_detailed_5/aggregate?drilldown%5B%5D=%5BYear%5D.%5BYear%5D&drilldown%5B%5D=%5BEnglish+Proficiency%5D.%5BEnglish+Proficiency%5D&drilldown%5B%5D=%5BNative+Language%5D.%5BNative+Language%5D&cut%5B%5D=%5BAdditive+Geography%5D.%5BTract%5D.%26%5B14000US36053030501%5D=&measures%5B%5D=Population+Sum&nonempty=true&distinct=false&parents=false&debug=false";
        let url = url::Url::parse(input).unwrap();
        for (k, v) in url.query_pairs() {
            println!("query_pairs: {}, {:?}", k, v);
        }
        assert_eq!(encode(input).unwrap(), correct_out.to_owned());
    }

    #[test]
    #[should_panic]
    fn test_encoding_bad_input() {
        let input = "hw chen/aggregate?one=[two =three].[four]&five=six";
        encode(input).unwrap();
    }

    #[test]
    fn test_byte_serialize() {
        let input = "&one";
        let correct_out= "%26one";
        assert_eq!(byte_serialize(input.as_bytes()).collect::<String>(), correct_out.to_owned());
    }
}
