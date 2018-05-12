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
`getrandom()`, `getentropy()`, `RtlGenRandom`, etc) via Rust's rand::[OsRng][osrng]
and sampled using its [range][range] API.

```
-% mkpass --help
mkpass 0.1.0
Thomas Hurst <tom@hur.st>
Generate reasonably secure passwords

USAGE:
    mkpass [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Activate verbose mode

OPTIONS:
    -b, --bits <bits>              Password strength target, 2^n [default: 72]
    -d, --dictionary <dict>        Built-in dictionary [default: eff]  [possible values: eff, diceware, beale, alpha,
                                   mixedalpha, mixedalphanumeric, alphanumeric, pin, hex, printable, koremutake]
    -l, --length <length>          Password length (overrides bits target)
    -n, --number <number>          Number of passwords to generate [default: 1]
    -s, --separator <separator>    Word separator
    -w, --wordlist <wordlist>      External dictionary
```

## Examples

```
# generate 5 passwords in verbose mode
-% mkpass -n 5 -v
# Complexity 7776^6=221073919720733357899776, 77.55 bits of entropy
carry pang flashing blouse mold antidote
blustery shrimp gag squire epidural zoology
mortuary banker roulette unplanned reproduce almost
tummy retake denial last superhero stifling
retiree diaper demystify igloo poem helmet
```

```
# generate a 128-bit passphrase from the system dictionary
-% mkpass -w /usr/share/dict/words -b 128
cleruchy fructose pierine catchpole espathate refigure kinbote nonpreformed
```

```
# generate a password using "koremutake" phonetics, with - as a separator
-% mkpass -d koremutake -s '-'
fri-vo-pu-tu-va-fre-fo-tre-dry-dri-ba
```

[eff]: https://www.eff.org/dice
[dice]: http://world.std.com/~reinhold/diceware.html
[kore]: http://shorl.com/koremutake.php
[osrng]: https://rust-num.github.io/num/rand/os/struct.OsRng.html
[range]: https://rust-num.github.io/num/rand/distributions/range/struct.Range.html
