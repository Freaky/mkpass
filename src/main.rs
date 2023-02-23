use std::collections::HashSet;
use std::path::PathBuf;

use clap::Parser;
use eyre::{ensure, eyre, Result, WrapErr};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::OsRng;
use read_restrict::read_to_string;

struct PassFormat {
    name: &'static str,
    data: &'static str,
    separator: &'static str,
    description: &'static str,
}

macro_rules! defdicts {
    ($($name:literal + $separator:literal = $description:literal)*) => {
        &[
            $(
                PassFormat {
                    name: $name,
                    data: include_str!(concat!("../dictionaries/", $name, ".txt")),
                    separator: $separator,
                    description: $description,
                },
            )*
        ]
    };
}

const DICTIONARIES: &[PassFormat] = defdicts! {
    "eff"               + " " = "EFF Long Wordlist\n  https://www.eff.org/dice"
    "eff-short1"        + " " = "EFF Short Wordlist - Fewer, shorter words"
    "eff-short2"        + " " = "EFF Short Wordlist - Fewer, longer words"
    "diceware"          + " " = "Arnold G. Reinhold's Diceware word list\n  https://theworld.com/~reinhold/diceware.html"
    "beale"             + " " = "Alan Beale's Diceware word list, \"contains fewer Americanisms and obscure words\""
    "alpha"             + ""  = "Lower-case a-z"
    "mixedalpha"        + ""  = "Mixed-case a-z"
    "mixedalphanumeric" + ""  = "Mixed-case a-z 0-9"
    "alphanumeric"      + ""  = "Lower-case a-z 0-9"
    "pin"               + ""  = "Numeric"
    "hex"               + ""  = "Hexadecimal"
    "printable"         + ""  = "Mixed-case a-z 0-9 plus standard ASCII symbols"
    "koremutake"        + "." = "A \"way to express any large number as a sequence of syllables\"\n  https://shorl.com/koremutake.php"
};

#[test]
fn test_dictionaries() {
    for dict in DICTIONARIES.iter() {
        assert!(dict.data.lines().count() > 1, "{} is too short", dict.name);

        assert!(
            dict.data.lines().all(|s| s == s.trim()),
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
    #[arg(short, long, default_value = "1", value_parser = clap::value_parser!(u32).range(1..))]
    number: u32,

    /// Password strength target, 2^n
    #[arg(short, long, default_value = "72")]
    bits: f64,

    /// Password length (overrides bits target)
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(1..))]
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

    /// Describe built-in dictionaries
    #[arg(short = 'D', long = "list-dictionaries")]
    list_dict: bool,
}

fn list_dictionaries() {
    for dict in DICTIONARIES {
        println!(
            "{}: {} entries\n  {}\n",
            dict.name,
            dict.data.lines().count(),
            dict.description
        );
    }
}

fn main() -> Result<()> {
    let opts = Opt::parse();
    let wordlist;
    let dict: Vec<&str>;
    let mut separator = " ";

    if let Some(wl) = opts.wordlist {
        wordlist = read_to_string(&wl, 1024 * 1024 * 128)
            .wrap_err_with(|| format!("Failed to read word list from {}", &wl.display()))?;
        dict = wordlist
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        ensure!(
            dict.len() > 2,
            eyre!("{}: dictionary too short", &wl.display())
        );
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

    if opts.list_dict {
        list_dictionaries();
        return Ok(());
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

    let mut random_words = Uniform::from(0..dict.len())
        .sample_iter(OsRng)
        .map(|i| dict[i]);

    for _ in 0..opts.number {
        let password = random_words
            .by_ref()
            .take(length as usize)
            .collect::<Vec<&str>>()
            .join(separator);
        println!("{}", password);
    }

    Ok(())
}
