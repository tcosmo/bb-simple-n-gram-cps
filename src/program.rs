#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub struct State(pub u8);

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                0 => "H",
                1 => "A",
                2 => "B",
                3 => "C",
                4 => "D",
                5 => "E",
                _ => todo!(),
            }
        )
    }
}

/**
 * A `Bit` is just a bool that appears in a tape.
 */
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub struct Bit(pub bool);

impl std::fmt::Display for Bit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if self.0 { '1' } else { '0' })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Dir {
    Left,
    Right,
}

impl Dir {
    pub fn opposite(self) -> Self {
        match self {
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
}

pub trait BitBlock: Sized {
    type FiveStorage;
    fn get_by(self, state: usize, storage: &Self::FiveStorage) -> &Option<(State, Self, Dir)>;
    fn get_by_mut(
        self,
        state: usize,
        storage: &mut Self::FiveStorage,
    ) -> &mut Option<(State, Self, Dir)>;
}

impl BitBlock for Bit {
    type FiveStorage = [Option<(State, Bit, Dir)>; 10];
    fn get_by(self, state: usize, storage: &Self::FiveStorage) -> &Option<(State, Self, Dir)> {
        &storage[state * 2 + (self.0 as usize)]
    }
    fn get_by_mut(
        self,
        state: usize,
        storage: &mut Self::FiveStorage,
    ) -> &mut Option<(State, Bit, Dir)> {
        &mut storage[state * 2 + (self.0 as usize)]
    }
}

#[derive(Debug)]
pub struct Program<Sym: BitBlock = Bit> {
    pub by_input_array: Sym::FiveStorage, // [Option<(State, Bit, Dir)>; 10], // HashMap<(Bit, State), (State, Bit, Dir)>,
}
impl<Sym: BitBlock + Clone> Program<Sym> {
    pub fn action(&self, read: Sym, state: State) -> Result<(State, Sym, Dir), MayHalt> {
        match Sym::get_by(read, (state.0 - 1) as usize, &self.by_input_array) {
            None => Err(MayHalt),
            Some(ans) => Ok(ans.clone()),
        }
    }
}

impl Program {
    pub fn from_string(s: &str) -> Program {
        if s.len() == 34 || s.len() == 30 {
            let s = s.as_bytes();
            let mut rules: Program<Bit> = Program {
                by_input_array: [None; 10],
            };

            fn color_from_char(c: u8) -> State {
                if c == b'A' || c == 1 {
                    return State(1);
                }
                if c == b'B' || c == 2 {
                    return State(2);
                }
                if c == b'C' || c == 3 {
                    return State(3);
                }
                if c == b'D' || c == 4 {
                    return State(4);
                }
                if c == b'E' || c == 5 {
                    return State(5);
                }
                panic!("unknown color {}", c);
            }
            fn bit_from_char(c: u8) -> Bit {
                if c == b'0' || c == 0 {
                    return Bit(false);
                }
                if c == b'1' || c == 1 {
                    return Bit(true);
                }
                panic!("unknown bit {}", c);
            }
            fn dir_from_char(c: u8) -> Dir {
                if c == b'R' || c == 0 {
                    return Dir::Right;
                }
                if c == b'L' || c == 1 {
                    return Dir::Left;
                }
                panic!("unknown dir {}", c);
            }

            for color in [
                (State(1), 0),
                (State(2), 1),
                (State(3), 2),
                (State(4), 3),
                (State(5), 4),
            ] {
                for bit in [(Bit(false), 0), (Bit(true), 1)] {
                    let i = color.1 * (if s.len() == 34 { 7 } else { 6 }) + bit.1 * 3;

                    if s[i + 2] == b'-' || s[i + 2] == 0 || s[i + 2] == b'Z' || s[i + 2] == b'H' {
                        // Halting state.
                        continue;
                    }

                    let conc = (
                        color_from_char(s[i + 2]),
                        bit_from_char(s[i]),
                        dir_from_char(s[i + 1]),
                    );

                    *Bit::get_by_mut(bit.0, color.1, &mut rules.by_input_array) = Some(conc);
                }
            }

            return rules;
        }

        panic!("unknown format, expected a 34-character string like '1RB0LC_0LA1RD_1LA0RB_1LE---_0RA1RE' or a 30-character string like '1RB0LC0LA1RD1LA0RB1LE---0RA1RE'");
    }
}

pub struct LoopsForever;
pub struct MayHalt;
