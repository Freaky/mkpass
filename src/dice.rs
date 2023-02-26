const COMMON_DICE: &[u32] = &[3, 4, 6, 8, 10, 12, 20, 30, 100];

#[derive(Debug, Copy, Clone)]
pub struct CandidateDice {
    pub sides: u32,
    pub rolls: u32,
    pub reroll_pct: f32,
    pub average_rolls: f32,
}

impl CandidateDice {
    pub fn from_sides_and_limit(sides: u32, limit: u32) -> Self {
        let mut rolls = 1;
        let mut total;
        loop {
            total = sides.pow(rolls);
            if total >= limit {
                break;
            }
            rolls += 1;
        }

        let overshoot = total - limit;
        let reroll_pct = overshoot as f32 / total as f32;

        Self {
            sides,
            rolls,
            reroll_pct,
            average_rolls: rolls as f32 + (rolls as f32 * reroll_pct),
        }
    }

    pub fn ordered_for_limit(limit: u32) -> Vec<Self> {
        let mut options: Vec<Self> = COMMON_DICE
            .iter()
            .copied()
            .map(|sides| Self::from_sides_and_limit(sides, limit))
            .collect();

        options.sort_by_key(|dice| (ordered_float::OrderedFloat(dice.average_rolls), dice.sides));
        options
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FastDiceRollerKind {
    Zero,
    Direct,
    PowerOfTwo,
    NonPowerOfTwo,
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
pub struct FastDiceRoller {
    modulus: u32,
    modulus_bits: f64,
    max_inclusive: u32,
    kind: FastDiceRollerKind,
}

impl FastDiceRoller {
    /// Generate values from 0..=max_inclusive from a uniform random source 0..modulus
    pub fn new(max_inclusive: u32, modulus: u32) -> Self {
        let modulus_bits = (modulus as f64).log2();

        let kind = if max_inclusive == 0 {
            FastDiceRollerKind::Zero
        } else if max_inclusive == modulus - 1 {
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
        }
    }

    fn read_dice(&self) -> u32 {
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

    fn power_of_two(&self) -> u32 {
        let mut x = 1;
        let mut y = 0;
        let mut next_bit = self.modulus_bits;
        let mut rngv = 0;

        loop {
            if next_bit >= self.modulus_bits {
                next_bit = 0.0;
                rngv = self.read_dice();
            }

            x *= 2;
            y = y * 2 + (rngv % 2);
            next_bit += 1.0;

            if x > self.max_inclusive {
                if y <= self.max_inclusive {
                    return y;
                }
                x = x - self.max_inclusive - 1;
                y = y - self.max_inclusive - 1;
            }
        }
    }

    fn non_power_of_two(&self) -> u32 {
        if self.max_inclusive < self.modulus {
            let n_plus_one = self.max_inclusive + 1;
            let max_exc = ((self.modulus - 1) / n_plus_one) * n_plus_one;
            loop {
                let ret = self.read_dice();
                if ret < n_plus_one {
                    return ret;
                }
                if ret < max_exc {
                    return ret % n_plus_one;
                }
            }
        } else {
            let cx = (self.max_inclusive / self.modulus) + 1;
            let mut cx_roller = FastDiceRoller::new(cx - 1, self.modulus);
            loop {
                let ret = cx * self.read_dice();
                if let Some(ret) = ret.checked_add(cx_roller.next().unwrap()) {
                    if ret <= self.max_inclusive {
                        return ret;
                    }
                }
            }
        }
    }
}

impl Iterator for FastDiceRoller {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.kind {
            FastDiceRollerKind::Zero => 0,
            FastDiceRollerKind::Direct => self.read_dice(),
            FastDiceRollerKind::PowerOfTwo => self.power_of_two(),
            FastDiceRollerKind::NonPowerOfTwo => self.non_power_of_two(),
        })
    }
}
