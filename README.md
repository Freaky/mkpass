# mkpass - make a passphrase

`mkpass` is a tool for securely generating passphrases in a preferred format.

By default it targets a minimum of 72-bits of entropy, corresponding to an
average cracking time of 75 years at 1 trillion guesses per second.

Bundled formats include:

* eff — [EFF Diceware][eff] (default)
* diceware — [Traditional Diceware][dice]
* beale — Alan Beale's Diceware (linked above)
* koremutake — [Shorl.com's Koremutake][kore]

Additionally a range of traditional non-word based formats are included for
generating alphanumeric passwords, PINs, etc, and custom word lists can also be
used.

Passwords are selected using the OS random number generator (`/dev/urandom`,
`getrandom()`, `getentropy()`, `RtlGenRandom`, etc) via Rust's
rand::[OsRng] (using [getrandom]) and sampled with its
[uniform distribution][uniform] API.

Alternatively you can use the `--dice` argument to use any dice with between 2
and 144 sides as a random number generator.  Rolls will be efficiently mapped to
passphrases using an implimentation of [this algorithm][Uniform Random Integers].

Diceware is designed for use with a d6, rolling five times per word.  This approach
allows for the use of, for example, d20 typically rolling just three times, or d100
rolling just twice.

Note this support is experimental.

## Usage

```
Generate reasonably secure passwords.

Use your operating system's cryptographic random number generator, or any dice
you have lying around, to create secure passwords without human bias.

Usage: mkpass [OPTIONS]

Options:
  -v, --verbose
          Activate verbose mode

  -s, --separator <SEPARATOR>
          Word separator

  -c, --count <COUNT>
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

      --dice <SIDES>
          Manually use dice for randomness.

  -D, --list-dictionaries
          Describe built-in dictionaries

      --dump
          Dump the selected dictionary to stdout and exit

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Examples

```
# generate 5 passphrases in verbose mode
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
# generate a 2-word EFF passphrase using a d20
-% mkpass --dice 20 -l 2
WARNING: Dice support is experimental.
Enter a dice roll, 1-20: 14
Enter a dice roll, 1-20: 6
Enter a dice roll, 1-20: 20
Enter a dice roll, 1-20: 11
Enter a dice roll, 1-20: 9
Enter a dice roll, 1-20: 1
referee paralyze
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
[OsRng]: https://docs.rs/rand/0.7.0/rand/rngs/struct.OsRng.html
[uniform]: https://docs.rs/rand/0.7.0/rand/distributions/struct.Uniform.html
[getrandom]: https://crates.io/crates/getrandom
[Uniform Random Integers]: https://peteroupc.github.io/randomfunc.html#RNDINT_Random_Integers_in_0_N