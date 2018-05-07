use std::fs::File;
use std::io::{BufRead, BufReader};

extern crate rand;
use rand::distributions::{IndependentSample, Range};

extern crate failure;
use failure::{Error, ResultExt};

#[macro_use]
extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;

fn sample_dict(dict: &Vec<String>, samples: usize) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let range = Range::new(0, dict.len());

    let mut ret: Vec<String> = Vec::new();

    for _ in 0..samples {
        ret.push(dict[range.ind_sample(&mut rng)].clone());
    }

    ret
}

#[derive(Debug, StructOpt)]
#[structopt(name = "mkpass", about = "Generate reasonably secure passwords")]
struct Opt {
    /// Activate verbose mode
    #[structopt(short = "v", long = "verbose")]
    debug: bool,
    /// Number of passwords to generate
    #[structopt(short = "n", long = "number", default_value = "1")]
    number: u32,
    /// Password strength target, 2^n
    #[structopt(short = "b", long = "bits", default_value = "64")]
    bits: f64,
    /// Password length (overrides bits target)
    #[structopt(short = "l", long = "length")]
    length: Option<u32>,
    /// Dictionary to use
    #[structopt(short = "w", long = "wordlist", parse(from_os_str))]
    wordlist: Option<PathBuf>,
}

fn run() -> Result<(), Error> {
    let opts = Opt::from_args();

    let mut wordlist: Vec<String> = vec![];
    if let Some(wl) = opts.wordlist {
        let inf = File::open(&wl).with_context(|e| format!("{}: {}", &wl.display(), e))?;
        let inf = BufReader::new(inf);

        wordlist.extend(inf.lines().map(|x| x.unwrap().to_lowercase()));
    } else {
        let eff = include_str!("../eff.txt");
        wordlist.extend(eff.lines().map(|x| x.to_lowercase()));
    }

    wordlist.sort_unstable();
    wordlist.dedup();

    let length;
    let bits;
    if let Some(l) = opts.length {
        length = l;
    } else {
        length = (opts.bits / (wordlist.len() as f64).log2()).ceil() as u32;
    }

    let combinations = (wordlist.len() as f64).powf(length as f64);
    bits = (combinations).log2();

    println!(
        "Complexity {}^{}={:.0}, {:.2} bits of entropy",
        wordlist.len(),
        length,
        combinations,
        bits
    );
    for _ in 0..opts.number {
        let pw = sample_dict(&wordlist, length as usize);
        println!("{}", pw.join(" "));
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("{}", e);
        std::process::exit(64);
    }
}
