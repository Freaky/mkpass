use ibig::UBig;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::OsRng;

#[derive(Debug, Clone, Copy, PartialEq)]
enum FastDiceRollerKind {
    Zero,
    Direct,
    PowerOfTwo,
    NonPowerOfTwo,
}

#[derive(Debug, Clone)]
pub struct FastDiceRoller {
    modulus: u32,
    modulus_bits: f64,
    max_inclusive: UBig,
    kind: FastDiceRollerKind,
    real_dice: bool,
}

/// A reformulation of Lumbroso's Fast Dice Roller described in 2009's
/// 'Optimal Discrete Uniform Generation from Coin Flips'
/// https://arxiv.org/pdf/1304.1916.pdf
///
/// Derived from pseudocode on Peter Occil's website:
/// https://peteroupc.github.io/randomfunc.html#RNDINT_Random_Integers_in_0_N
///
/// This Twitter thread provides some further insight into the algorithm:
/// https://twitter.com/gro_tsen/status/1386448258176884737
impl FastDiceRoller {
    /// Generate values from 0..=max_inclusive from a uniform random source 0..modulus
    pub fn new(max_inclusive: UBig, modulus: u32, real_dice: bool) -> Self {
        let modulus_bits = (modulus as f64).log2();

        let kind = if max_inclusive == UBig::from(0u32) {
            FastDiceRollerKind::Zero
        } else if max_inclusive == (modulus - 1).into() {
            FastDiceRollerKind::Direct
        } else if modulus_bits.floor() == modulus_bits {
            FastDiceRollerKind::PowerOfTwo
        } else {
            FastDiceRollerKind::NonPowerOfTwo
        };

        Self {
            modulus,
            modulus_bits,
            max_inclusive,
            kind,
            real_dice,
        }
    }

    fn read_dice(&self) -> u32 {
        if self.real_dice {
            self.read_real_dice()
        } else {
            Uniform::from(0..self.modulus).sample(&mut OsRng)
        }
    }

    fn read_real_dice(&self) -> u32 {
        let mut rl = rustyline::DefaultEditor::new().unwrap();
        let prompt = format!("Enter a dice roll, 1-{}: ", self.modulus);
        loop {
            if let Ok(line) = rl.readline(&prompt) {
                if let Ok(num) = line.trim().parse::<u32>() {
                    if num > 0 && num <= self.modulus {
                        return num - 1;
                    }
                    eprintln!("{} out of range 1-{}", num, self.modulus);
                } else {
                    eprintln!("Parse error. Try again or ^C to cancel.");
                }
            } else {
                std::process::exit(1);
            }
        }
    }

    fn power_of_two(&self) -> UBig {
        let mut x = UBig::from(1u32);
        let mut y = UBig::from(0u32);
        let mut next_bit = self.modulus_bits;
        let mut rngv = 0;

        loop {
            if next_bit >= self.modulus_bits {
                next_bit = 0.0;
                rngv = self.read_dice();
            }

            x *= 2;
            y = y * 2 + (rngv % 2);
            rngv /= 2;
            next_bit += 1.0;

            if x > self.max_inclusive {
                if y <= self.max_inclusive {
                    return y;
                }
                x = x - &self.max_inclusive - 1;
                y = y - &self.max_inclusive - 1;
            }
        }
    }

    fn non_power_of_two(&self) -> UBig {
        if self.max_inclusive < UBig::from(self.modulus) {
            let n_plus_one = &self.max_inclusive + 1;
            let max_exc = ((UBig::from(self.modulus - 1)) / &n_plus_one) * &n_plus_one;
            loop {
                let ret = UBig::from(self.read_dice());
                if ret < n_plus_one {
                    return ret;
                }
                if ret < max_exc {
                    return ret % n_plus_one;
                }
            }
        } else {
            let cx = (&self.max_inclusive / self.modulus) + 1;
            let mut cx_roller = FastDiceRoller::new(&cx - 1, self.modulus, self.real_dice);
            loop {
                let mut ret: UBig = &cx * self.read_dice();
                ret += cx_roller.next().unwrap();
                if ret <= self.max_inclusive {
                    return ret;
                }
            }
        }
    }
}

impl Iterator for FastDiceRoller {
    type Item = UBig;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.kind {
            FastDiceRollerKind::Zero => UBig::from(0u32),
            FastDiceRollerKind::Direct => UBig::from(self.read_dice()),
            FastDiceRollerKind::PowerOfTwo => self.power_of_two(),
            FastDiceRollerKind::NonPowerOfTwo => self.non_power_of_two(),
        })
    }
}
