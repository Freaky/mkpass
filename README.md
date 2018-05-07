# mkpass - make a password

`mkpass` is a simple tool for generating passwords from wordlists.

It will target a minimum of 72 bits of entropy, which corresponds to an average
cracking time of 75 years at 1 trillion guesses per second.

It includes a bundled copy of the [EFF Diceware wordlist][1], which will be used
as the default dictionary if another wordlist isn't provided.

```
-% mkpass --help
mkpass 0.1.0
Thomas Hurst <tom@hur.st>
Generate reasonably secure passwords

USAGE:
    mkpass [FLAGS] [OPTIONS]

FLAGS:
    -v, --verbose    Activate verbose mode
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --bits <bits>            Password strength target, 2^n [default: 72]
    -l, --length <length>        Password length (overrides bits target)
    -n, --number <number>        Number of passwords to generate [default: 1]
    -w, --wordlist <wordlist>    Dictionary to use
```

## Examples

```
-% mkpass -n 5 -v # generate 10 passwords in verbose mode
# Complexity 7776^6=221073919720733357899776, 77.55 bits of entropy
carry pang flashing blouse mold antidote
blustery shrimp gag squire epidural zoology
mortuary banker roulette unplanned reproduce almost
tummy retake denial last superhero stifling
retiree diaper demystify igloo poem helmet
```

```
-% mkpass -b 128 # generate a 128-bit passphrase from the system dictionary
cleruchy fructose pierine catchpole espathate refigure kinbote nonpreformed
```

[1]: https://www.eff.org/dice
