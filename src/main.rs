use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::PathBuf;

use lazy_static::lazy_static;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::OsRng;
use clap::Parser;

struct PassFormat {
    name: &'static str,
    data: &'static str,
    separator: &'static str,
}

macro_rules! defdicts {
    ($($name:expr => $separator:expr)*) => {
        vec![
            $(
                PassFormat {
                    name: $name,
                    data: include_str!(concat!("../dictionaries/", $name, ".txt")),
                    separator: $separator,
                },
            )*
        ]
    };
}

lazy_static! {
    static ref DICTIONARIES: Vec<PassFormat> = defdicts! {
        "eff"               => " "
        "eff-short1"        => " "
        "eff-short2"        => " "
        "diceware"          => " "
        "beale"             => " "
        "alpha"             => ""
        "mixedalpha"        => ""
        "mixedalphanumeric" => ""
        "alphanumeric"      => ""
        "pin"               => ""
        "hex"               => ""
        "printable"         => ""
        "koremutake"        => "."
    };
}

#[test]
fn test_dictionaries() {
    for dict in DICTIONARIES.iter() {
        assert!(dict.data.lines().count() > 1, "{} is too short", dict.name);

        assert!(
            dict.data.lines().all(|s| &s[..] == s.trim()),
            "leading/trailing whitespace in {}",
            dict.name
        );

        assert!(
            !dict.data.lines().any(str::is_empty),
            "blank line in {}",
            dict.name
        );

        assert_eq!(
            dict.data.lines().count(),
            dict.data.lines().collect::<HashSet<_>>().len(),
            "duplicate entry in {}",
            dict.name
        );
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about = "Generate reasonably secure passwords")]
struct Opt {
    /// Activate verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Word separator
    #[arg(short, long)]
    separator: Option<String>,

    /// Number of passwords to generate
    #[arg(short, long, default_value = "1")]
    number: u32,

    /// Password strength target, 2^n
    #[arg(short, long, default_value = "72")]
    bits: f64,

    /// Password length (overrides bits target)
    #[arg(short, long)]
    length: Option<u32>,

    /// External dictionary
    #[arg(short, long, value_name = "FILE")]
    wordlist: Option<PathBuf>,

    /// Built-in dictionary
    #[arg(
        short,
        long = "dictionary",
        default_value = "eff",
        value_parser = clap::builder::PossibleValuesParser::new(&DICTIONARIES.iter().map(|s| s.name).collect::<Vec<&str>>())
    )]
    dict: String,
}

fn run() -> Result<(), String> {
    let opts = Opt::parse();
    let wordlist;
    let dict: Vec<&str>;
    let mut separator = " ";

    if let Some(wl) = opts.wordlist {
        wordlist = read_to_string(&wl).map_err(|e| format!("{}: {}", &wl.display(), e))?;
        dict = wordlist
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        if dict.len() < 2 {
            return Err(format!("{}: dictionary too short", &wl.display()));
        }
    } else {
        let d = DICTIONARIES
            .iter()
            .find(|x| x.name == opts.dict)
            .expect("Can't find dictionary");
        dict = d.data.lines().collect();
        separator = d.separator;
    }

    if let Some(ref s) = opts.separator {
        separator = s;
    }

    let length = opts
        .length
        .unwrap_or((opts.bits / (dict.len() as f64).log2()).ceil() as u32);

    if opts.verbose {
        let combinations = (dict.len() as f64).powf(f64::from(length));
        println!(
            "# Complexity {}^{}={:.0}, {:.2} bits of entropy",
            dict.len(),
            length,
            combinations,
            combinations.log2()
        );
    }

    let mut sampler = Uniform::from(0..dict.len()).sample_iter(OsRng);
    for _ in 0..opts.number {
        let pw = sampler
            .by_ref()
            .take(length as usize)
            .map(|i| dict[i])
            .collect::<Vec<&str>>()
            .join(&separator);
        println!("{}", pw);
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(64);
    }
}
