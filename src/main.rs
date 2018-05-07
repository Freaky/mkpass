use std::fs::File;
use std::io::Read;

extern crate rand;
use rand::distributions::{IndependentSample, Range};

extern crate failure;
use failure::{Error, ResultExt};

#[macro_use]
extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;

fn sample_dict<'a>(mut rng: &mut rand::OsRng, dict: &'a [&str], samples: usize) -> Vec<&'a str> {
    let range = Range::new(0, dict.len());

    let mut ret: Vec<&str> = Vec::new();

    for _ in 0..samples {
        ret.push(dict[range.ind_sample(&mut rng)]);
    }

    ret
}

#[derive(Debug, StructOpt)]
#[structopt(name = "mkpass", about = "Generate reasonably secure passwords")]
struct Opt {
    /// Activate verbose mode
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Word separator
    #[structopt(short = "s", long = "separator", default_value = " ")]
    separator: String,

    /// Number of passwords to generate
    #[structopt(short = "n", long = "number", default_value = "1")]
    number: u32,

    /// Password strength target, 2^n
    #[structopt(short = "b", long = "bits", default_value = "72")]
    bits: f64,

    /// Password length (overrides bits target)
    #[structopt(short = "l", long = "length")]
    length: Option<u32>,

    /// Dictionary to use (default: built-in EFF Diceware)
    #[structopt(short = "w", long = "wordlist", parse(from_os_str))]
    wordlist: Option<PathBuf>,
}

fn run() -> Result<(), Error> {
    let opts = Opt::from_args();
    let eff = include_str!("../eff.txt");

    let mut dict: Vec<&str> = vec![];

    if let Some(wl) = opts.wordlist {
        let mut wordlist = String::new();
        let mut inf = File::open(&wl).with_context(|e| format!("{}: {}", &wl.display(), e))?;
        inf.read_to_string(&mut wordlist)
            .with_context(|e| format!("{}: {}", &wl.display(), e))?;
        dict.extend(wordlist.lines());
    } else {
        dict.extend(eff.lines());
    }

    dict.sort_unstable();
    dict.dedup();

    let length;
    let bits;
    if let Some(l) = opts.length {
        length = l;
    } else {
        length = (opts.bits / (dict.len() as f64).log2()).ceil() as u32;
    }

    let combinations = (dict.len() as f64).powf(f64::from(length));
    bits = (combinations).log2();

    let mut rng = rand::OsRng::new().expect("Failed to open RNG");

    if opts.verbose {
        println!(
            "# Complexity {}^{}={:.0}, {:.2} bits of entropy",
            dict.len(),
            length,
            combinations,
            bits
        );
    }
    for _ in 0..opts.number {
        let pw = sample_dict(&mut rng, &dict, length as usize);
        println!("{}", pw.join(&opts.separator));
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("{}", e);
        std::process::exit(64);
    }
}
