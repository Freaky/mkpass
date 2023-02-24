#![allow(clippy::uninlined_format_args)]

use std::collections::HashSet;
use std::path::PathBuf;

use clap::Parser;
use eyre::{ensure, eyre, Result, WrapErr};
use ibig::UBig;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::OsRng;
use read_restrict::read_to_string;

#[derive(Debug)]
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
    "koremutake"        + " " = "A \"way to express any large number as a sequence of syllables\"\n  https://shorl.com/koremutake.php"
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

fn crack_times(combinations: &UBig) -> Vec<(&'static str, UBig)> {
    vec![
        ("Online, unthrottled (10/s)", combinations / 10),
        ("Online, throttled (1/s)", combinations.clone()),
        ("Offline, slow (1e4/s)", combinations / 1000),
        ("Offline, fast (1e10/s)", combinations / 10_000_000_000u64),
        (
            "Offline, extreme (1e12/s)",
            combinations / 1_000_000_000_000u64,
        ),
    ]
}

fn password_strength(entropy: u32) -> &'static str {
    const THRESHOLDS: &[(u32, &str)] = &[
        (29, "very weak"),
        (36, "weak"),
        (49, "somewhat weak"),
        (60, "reasonable"),
        (70, "strong"),
        (127, "very strong"),
        (256, "cryptographic"),
    ];

    THRESHOLDS
        .iter()
        .find(|(thresh, _)| entropy < *thresh)
        .map_or("overkill", |(_, desc)| *desc)
}

fn human_duration(secs: UBig) -> String {
    let thresholds: &[(UBig, &str, &str)] = &[
        (60u32.into(), "minute", "minutes"),
        (24u32.into(), "hour", "hours"),
        (30u32.into(), "day", "days"),
        (12u32.into(), "month", "months"),
        (10u32.into(), "year", "years"),
        (10u32.into(), "decade", "decades"),
        (10u32.into(), "century", "centuries"),
        (1000u32.into(), "millennium", "millennia"),
        (1000u32.into(), "million year", "million years"),
        (1000u32.into(), "billion year", "billion years"),
    ];

    if secs < 1u32.into() {
        return "less than a second".to_string();
    } else if secs < 60u32.into() {
        return "less than a minute".to_string();
    }

    let mut interval: UBig = secs / 60;
    for (divisor, single, plural) in thresholds {
        if interval < *divisor {
            return format!(
                "{} {}",
                interval,
                if interval == UBig::from(1u32) {
                    single
                } else {
                    plural
                }
            );
        }
        interval /= divisor;
    }

    "trillions of years".to_string()
}

/// Generate reasonably secure passwords.
///
/// Uses the OS standard cryptographic random number generator to generate
/// passwords without human bias.
#[derive(Debug, Parser)]
#[command(author, version)]
struct Opt {
    /// Activate verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Word separator
    #[arg(short, long)]
    separator: Option<String>,

    /// Number of passwords to generate
    #[arg(short, short_alias = 'c', long, default_value = "1", value_parser = clap::value_parser!(u32).range(1..))]
    number: u32,

    /// Password strength target, 2^n
    #[arg(short, long, default_value = "72")]
    bits: f64,

    /// Password length (overrides bits target)
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(1..65535))]
    length: Option<u32>,

    /// External dictionary, line-separated
    #[arg(short, long, value_name = "PATH")]
    file: Option<PathBuf>,

    /// Built-in dictionary
    #[arg(
        short,
        short_alias = 'w',
        long,
        default_value = "eff",
        value_parser = clap::builder::PossibleValuesParser::new(&DICTIONARIES.iter().map(|s| s.name).collect::<Vec<&str>>())
    )]
    dictionary: String,

    /// Describe built-in dictionaries
    #[arg(short = 'D', long)]
    list_dictionaries: bool,
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

    if opts.list_dictionaries {
        list_dictionaries();
        return Ok(());
    }

    if let Some(wl) = opts.file {
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

        if opts.verbose {
            eprintln!("# {:>12}: {}", "Dictionary", wl.display());
        }
    } else {
        let d = DICTIONARIES
            .iter()
            .find(|x| x.name == opts.dictionary)
            .expect("Can't find dictionary");
        dict = d.data.lines().collect();
        separator = d.separator;

        if opts.verbose {
            eprintln!("# {:>12}: {}", "Dictionary", opts.dictionary);
            eprintln!(
                "# {:>12}: {}",
                "Description",
                d.description.replace('\n', "")
            );
        }
    }

    if let Some(ref s) = opts.separator {
        separator = s;
    }

    let length = opts
        .length
        .unwrap_or((opts.bits / (dict.len() as f64).log2()).ceil() as u32);

    if opts.verbose {
        let combinations = UBig::from(dict.len()).pow(length as usize);
        let entropy = (dict.len() as f64).log2() * length as f64;
        eprintln!(
            "# {:>12}: {}^{} = {:.0}",
            "Combinations",
            dict.len(),
            length,
            combinations,
        );
        eprintln!(
            "# {:>12}: {:.2} bits ({})",
            "Entropy",
            entropy,
            password_strength(entropy as u32)
        );
        eprintln!("#");
        eprintln!("# Attack time estimate:");
        for (attack, duration) in crack_times(&combinations) {
            eprintln!("# {:>28}: {}", attack, human_duration(duration));
        }
        eprintln!("#");
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
