#![allow(clippy::uninlined_format_args)]

use std::collections::HashSet;
use std::path::PathBuf;

use clap::{builder::PossibleValuesParser, Parser};
use eyre::{ensure, eyre, Result, WrapErr};
use ibig::UBig;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::OsRng;
use read_restrict::read_to_string;

mod dice;
use dice::FastDiceRoller;

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

fn crack_times(combinations: &UBig) -> Vec<(&'static str, f64)> {
    vec![
        (
            "Online, throttled (100/h)",
            combinations.to_f64() / (100.0 / 3600.0),
        ),
        (
            "Online, unthrottled (10/s)",
            (combinations / 10u32).to_f64(),
        ),
        ("Offline, slow (1e4/s)", (combinations / 1000u32).to_f64()),
        (
            "Offline, fast (1e10/s)",
            (combinations / 10_000_000_000u64).to_f64(),
        ),
        (
            "Offline, extreme (1e12/s)",
            (combinations / 1_000_000_000_000u64).to_f64(),
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

fn human_duration(secs: f64) -> String {
    let thresholds: &[(f64, &str, &str)] = &[
        (60.0, "minute", "minutes"),
        (24.0, "hour", "hours"),
        (30.437, "day", "days"),
        (12.0, "month", "months"),
        (10.0, "year", "years"),
        (10.0, "decade", "decades"),
        (10.0, "century", "centuries"),
        (1000.0, "millennium", "millennia"),
        (1000.0, "million year", "million years"),
        (1000.0, "billion year", "billion years"),
    ];

    if secs < 1.0 {
        return "less than a second".to_string();
    } else if secs < 60.0 {
        return "less than a minute".to_string();
    }

    let mut interval = secs / 60.0;
    for (divisor, single, plural) in thresholds {
        if interval < *divisor {
            let rounded = interval.round() as u64;
            return format!("{} {}", rounded, if rounded == 1 { single } else { plural });
        }
        interval /= divisor;
    }

    "trillions of years".to_string()
}

fn parse_target_bits(arg: &str) -> Result<f64, &'static str> {
    match arg.parse::<f64>() {
        Ok(f) if f.is_finite() && (1.0..65535.0).contains(&f) => Ok(f),
        Ok(_) => Err("Not within range 1..65535"),
        Err(_) => Err("Not a number"),
    }
}

/// Generate reasonably secure passwords.
///
/// Use your operating system's cryptographic random number generator, or any
/// dice you have lying around, to create secure passwords without human bias.
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
    #[arg(short, short_alias = 'n', long, default_value = "1")]
    count: u32,

    /// Password strength target, 2^n
    #[arg(short, long, default_value_t = 72.0, value_parser = parse_target_bits)]
    bits: f64,

    /// Password length (overrides bits target)
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(1..65535))]
    length: Option<u32>,

    /// External dictionary, line-separated
    #[arg(short, long, value_name = "PATH", value_parser = clap::value_parser!(PathBuf))]
    file: Option<PathBuf>,

    /// Built-in dictionary
    #[arg(
        short,
        short_alias = 'w',
        long,
        default_value = "eff",
        value_parser = PossibleValuesParser::new(DICTIONARIES.iter().map(|s| s.name))
    )]
    dictionary: String,

    /// Manually use dice for randomness.
    #[arg(
        long,
        value_name = "SIDES",
        value_parser = clap::value_parser!(u32).range(2..145)
    )]
    dice: Option<u32>,

    /// Describe built-in dictionaries
    #[arg(short = 'D', long)]
    list_dictionaries: bool,

    /// Dump the selected dictionary to stdout and exit
    #[arg(long = "dump")]
    dump: bool,
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

    if opts.dump {
        for word in &dict {
            println!("{}", word);
        }
        return Ok(());
    }

    if let Some(ref s) = opts.separator {
        separator = s;
    }

    let bits_per_word = (dict.len() as f64).log2();
    let length = opts
        .length
        .unwrap_or((opts.bits / bits_per_word).ceil() as u32)
        .max(1);

    if opts.verbose {
        let combinations = UBig::from(dict.len()).pow(length as usize);
        let entropy = bits_per_word * length as f64;
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

    let mut random_words: Box<dyn Iterator<Item = &str>> = if let Some(sides) = opts.dice {
        eprintln!("WARNING: Dice support is experimental.");
        let dice =
            FastDiceRoller::new(UBig::from(dict.len()).pow(length as usize) - 1, sides, true);
        Box::new(dice.flat_map(|roll| {
            let dict = &dict;
            (0..length).rev().map(move |i| {
                let idx = if i > 0 {
                    (&roll / (UBig::from(dict.len()).pow(i as usize))) % dict.len()
                } else {
                    &roll % dict.len()
                };
                dict[idx]
            })
        }))
    } else {
        Box::new(
            Uniform::from(0..dict.len())
                .sample_iter(OsRng)
                .map(|i| dict[i]),
        )
    };

    for _ in 0..opts.count {
        let password = random_words
            .by_ref()
            .take(length as usize)
            .collect::<Vec<&str>>()
            .join(separator);
        println!("{}", password);
    }

    Ok(())
}
