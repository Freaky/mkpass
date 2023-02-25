pub struct DiceRollRandom {
    modulus: u32,
}

impl DiceRollRandom {
    pub fn new(modulus: u32) -> Self {
        Self { modulus }
    }

    pub fn gen(&self, limit: u32) -> u32 {
        rand_int(limit - 1, self.modulus)
    }
}

fn read_dice(modulus: u32) -> u32 {
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    let prompt = format!("Enter a dice roll, 1-{}: ", modulus);
    loop {
        if let Ok(line) = rl.readline(&prompt) {
            if let Ok(num) = line.trim().parse::<u32>() {
                if num > 0 && num <= modulus {
                    return num - 1;
                }
                eprintln!("{} out of range 1-{}", num, modulus);
            } else {
                eprintln!("Parse error. Try again or ^C to cancel.");
            }
        } else {
            std::process::exit(1);
        }
    }
}

// Translated more or less directly from
// https://peteroupc.github.io/randomfunc.html#RNDINT_Random_Integers_in_0_N
fn rand_int_helper_non_power_of_two(max_inclusive: u32, modulus: u32) -> u32 {
    if max_inclusive <= modulus - 1 {
        let n_plus_one = max_inclusive + 1;
        let max_exc = ((modulus - 1) / n_plus_one) * n_plus_one;
        loop {
            let ret = read_dice(modulus);
            if ret < n_plus_one {
                return ret;
            }
            if ret < max_exc {
                return ret % n_plus_one;
            }
        }
    } else {
        let cx = (max_inclusive / modulus) + 1;
        loop {
            let mut ret = cx * read_dice(modulus);
            ret = ret.checked_add(rand_int(cx - 1, modulus)).unwrap();
            if ret <= max_inclusive {
                return ret;
            }
        }
    }
}

fn rand_int_helper_power_of_two(max_inclusive: u32, modulus: u32) -> u32 {
    let mod_bits = (modulus as f64).log2();
    let mut x = 1;
    let mut y = 0;
    let mut next_bit = mod_bits;
    let mut rngv = 0;

    loop {
        if next_bit >= mod_bits {
            next_bit = 0.0;
            rngv = read_dice(modulus);
        }

        x *= 2;
        y = y * 2 + (rngv % 2);
        next_bit += 1.0;

        if x > max_inclusive {
            if y <= max_inclusive {
                return y;
            }
            x = x - max_inclusive - 1;
            y = y - max_inclusive - 1;
        }
    }
}

fn rand_int(max_inclusive: u32, modulus: u32) -> u32 {
    if max_inclusive == 0 {
        return 0;
    }

    if max_inclusive == modulus - 1 {
        return read_dice(modulus);
    }

    let mod_bits = (modulus as f64).log2();
    if mod_bits.floor() == mod_bits {
        rand_int_helper_power_of_two(max_inclusive, modulus)
    } else {
        rand_int_helper_non_power_of_two(max_inclusive, modulus)
    }
}
