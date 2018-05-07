# mkpass - make a password

`mkpass` is a simple tool for generating passwords from wordlists.

It will target a minimum of 72 bits of entropy, which corresponds to an average
cracking time of 75 years at 1 trillion guesses per second.

It includes a bundled copy of the [EFF Diceware wordlist][1], which will be used
as the default dictionary if another wordlist isn't provided.

Passwords are selected using the OS random number generator (`/dev/urandom`,
`getrandom()`, `getentropy()`, `RtlGenRandom`, etc) via Rust's rand::[OsRng][2]
and sampled using its [range][3] API.

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
    -l, --length <length>          Password length (overrides bits target)
    -n, --number <number>          Number of passwords to generate [default: 1]
    -s, --separator <separator>    Word separator [default:  ]
    -w, --wordlist <wordlist>      Dictionary to use (default: built-in EFF Diceware)
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

[1]: https://www.eff.org/dice
[2]: https://rust-num.github.io/num/rand/os/struct.OsRng.html
[3]: https://rust-num.github.io/num/rand/distributions/range/struct.Range.html
