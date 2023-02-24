# mkpass - make a password

`mkpass` is a simple tool for generating passwords from dictionaries.

It will target a minimum of 72 bits of entropy, which corresponds to an average
cracking time of 75 years at 1 trillion guesses per second.

Bundled wordlists include:

* eff — [EFF Diceware][eff] (default)
* diceware — [Traditional Diceware][dice]
* beale — Alan Beale's Diceware (linked above)
* koremutake — [Shorl.com's Koremutake][kore]

Passwords are selected using the OS random number generator (`/dev/urandom`,
`getrandom()`, `getentropy()`, `RtlGenRandom`, etc) via Rust's
rand::[OsRng][osrng] (using [getrandom]) and sampled with its
[uniform distribution][uniform] API.

```
-% mkpass --help
Generate reasonably secure passwords.

Uses the OS standard cryptographic random number generator to generate passwords
without human bias.

Usage: mkpass [OPTIONS]

Options:
  -v, --verbose
          Activate verbose mode

  -s, --separator <SEPARATOR>
          Word separator

  -n, --number <NUMBER>
          Number of passwords to generate

          [default: 1]

  -b, --bits <BITS>
          Password strength target, 2^n

          [default: 72]

  -l, --length <LENGTH>
          Password length (overrides bits target)

  -f, --file <PATH>
          External dictionary, line-separated

  -d, --dictionary <DICTIONARY>
          Built-in dictionary

          [default: eff]
          [possible values: eff, eff-short1, eff-short2, diceware, beale, alpha,
          mixedalpha, mixedalphanumeric, alphanumeric, pin, hex, printable,
          koremutake]

  -D, --list-dictionaries
          Describe built-in dictionaries

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Examples

```
# generate 5 passwords in verbose mode
-% mkpass -n 5 -v
#   Dictionary: eff
#  Description: EFF Long Wordlist  https://www.eff.org/dice
# Combinations: 7776^6 = 221073919720733357899776
#      Entropy: 77.55 bits (very strong)
#
# Attack time estimate:
#   Online, unthrottled (10/s): trillions of years
#    Online, throttled (100/h): trillions of years
#        Offline, slow (1e4/s): 701 billion years
#       Offline, fast (1e10/s): 701 minnennia
#    Offline, extreme (1e12/s): 7 minnennia
#
skyline skimming vacant removable reunion critter
sudden ungloved footsie spectrum vision sixtieth
data husband wobbling enroll ultra pacifier
clay glamorous unnamed blast jockey astrology
shakily sprint crafty mortuary kept nanometer
```

```
# generate a 128-bit passphrase from the system dictionary
-% mkpass -f /usr/share/dict/words -b 128
cleruchy fructose pierine catchpole espathate refigure kinbote nonpreformed
```

```
# generate a password using "koremutake" phonetics, with - as a separator
-% mkpass -d koremutake -s -
fri-vo-pu-tu-va-fre-fo-tre-dry-dri-ba
```

[eff]: https://www.eff.org/dice
[dice]: http://world.std.com/~reinhold/diceware.html
[kore]: http://shorl.com/koremutake.php
[osrng]: https://docs.rs/rand/0.7.0/rand/rngs/struct.OsRng.html
[uniform]: https://docs.rs/rand/0.7.0/rand/distributions/struct.Uniform.html
[getrandom]: https://crates.io/crates/getrandom