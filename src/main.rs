use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

extern crate rand;
use rand::distributions::{IndependentSample, Range};

#[macro_use]
extern crate failure;
use failure::{Error, ResultExt};

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate structopt;

use structopt::StructOpt;

struct PassFormat {
    name: &'static str,
    data: &'static str,
    separator: String,
}

macro_rules! defdict {
    ($vec:expr, $name:expr, $separator:expr) => {
        $vec.push(PassFormat {
            name: $name,
            data: include_str!(concat!("../dictionaries/", $name, ".txt")),
            separator: $separator.to_owned(),
        });
    };
}

lazy_static! {
    static ref DICTIONARIES: Vec<PassFormat> = {
        let mut m = Vec::with_capacity(10);
        defdict!(m, "eff", " ");
        defdict!(m, "alpha", "");
        defdict!(m, "mixedalpha", "");
        defdict!(m, "mixedalphanumeric", "");
        defdict!(m, "alphanumeric", "");
        defdict!(m, "pin", "");
        defdict!(m, "hex", "");
        defdict!(m, "printable", "");
        defdict!(m, "koremutake", "");
        m
    };
}

#[derive(Debug, StructOpt)]
#[structopt(name = "mkpass", about = "Generate reasonably secure passwords")]
struct Opt {
    /// Activate verbose mode
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Word separator
    #[structopt(short = "s", long = "separator")]
    separator: Option<String>,

    /// Number of passwords to generate
    #[structopt(short = "n", long = "number", default_value = "1")]
    number: u32,

    /// Password strength target, 2^n
    #[structopt(short = "b", long = "bits", default_value = "72")]
    bits: f64,

    /// Password length (overrides bits target)
    #[structopt(short = "l", long = "length")]
    length: Option<u32>,

    /// External dictionary
    #[structopt(short = "w", long = "wordlist", parse(from_os_str))]
    wordlist: Option<PathBuf>,

    /// Built-in dictionary [default: eff]
    #[structopt(short = "d", long = "dictionary",
                raw(possible_values = "&DICTIONARIES.iter().map(|s| s.name).collect::<Vec<&str>>()"))]
    dict: Option<String>,
}

fn sample_dict<'a>(mut rng: &mut rand::OsRng, dict: &'a [&str], samples: usize) -> Vec<&'a str> {
    let range = Range::new(0, dict.len());

    let mut ret: Vec<&str> = Vec::new();

    for _ in 0..samples {
        ret.push(dict[range.ind_sample(&mut rng)]);
    }

    ret
}

fn run() -> Result<(), Error> {
    let opts = Opt::from_args();
    let eff = include_str!("../eff.txt");
    let mut wordlist = String::new();
    let mut dict: Vec<&str>;
    let mut separator = " ".to_owned();

    if let Some(wl) = opts.wordlist {
        let mut inf = File::open(&wl).with_context(|e| format!("{}: {}", &wl.display(), e))?;
        inf.read_to_string(&mut wordlist)
            .with_context(|e| format!("{}: {}", &wl.display(), e))?;
        dict = wordlist
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        dict.sort_unstable();
        dict.dedup();
    } else if let Some(d) = opts.dict {
        let dd = DICTIONARIES
            .iter()
            .find(|x| x.name == &d[..])
            .expect("Can't find dictionary");
        dict = dd.data.lines().collect();
        separator = dd.separator.clone();
    } else {
        dict = eff.lines().collect();
    }

    let length = opts.length
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

    let mut rng = rand::OsRng::new().expect("Failed to open RNG");
    for _ in 0..opts.number {
        let pw = sample_dict(&mut rng, &dict, length as usize);
        println!("{}", pw.join(&separator));
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("{}", e);
        std::process::exit(64);
    }
}
