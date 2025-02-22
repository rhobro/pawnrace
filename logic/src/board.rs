use std::{env, fmt::Display, str::FromStr};
use anyhow::anyhow;

use crate::IO;

pub fn test(mut io: IO) {
    let b = Board::new().flip();
    io.send(b.to_string());

    dbg!(b.at(('A', 1)));
    dbg!(b.at(('A', 2)));
    dbg!(b.at(('A', 3)));
    dbg!(b.at(('B', 2)));

    dbg!(b.at(('A', 6)));
    dbg!(b.at(('A', 7)));
    dbg!(b.at(('A', 8)));
    dbg!(b.at(('B', 7)));
}

// Storing chessboard board as bits
// Assuming white starts from bottom
// 2 bits per square:
//  - white: 01
//  - black: 10
//  - empty: 00, 11
// Using this config so that reversing bits reverses colours
#[derive(Clone, Copy)]
struct Board {
    raw: u128,
}

impl Board {
    const fn new() -> Self {
        Self {
            // full board
            raw: 0x0000AAAA000000000000000055550000,
        }
    }

    const fn flip(&self) -> Self {
        Self {
            raw: self.raw.reverse_bits(),
        }
    }

    fn at(&self, pos: impl Into<Position>) -> Square {
        let pos = pos.into();

        let bit_pos = 2 * (8 * pos.rank.0 + pos.file.0);
        let mask: u128 = 0b11 << bit_pos;
        let v = (self.raw & mask) >> bit_pos;
        Square::decode(v as usize)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "    -----------------\n")?;

        for rank in (1..9).rev() {
            let rank = Rank::new(rank);

            // start row
            write!(f, " {rank} |")?;
            // fill pieces
            for file in 1..9 {
                let file = File::new(file as u32);
                let pos = Position::new(file, rank);

                write!(f, " {}", self.at(pos))?;
            }
            // end row
            write!(f, "\n")?;
        }

        // end
        write!(f, "    -----------------\n")?;
        write!(f, "     A B C D E F G H\n")
    }
}

#[derive(Clone, Copy, Debug)]
enum Square {
    Piece(Colour),
    Empty,
}

impl Square {
    fn decode(v: usize) -> Self {
        match v {
            0b01 => Square::Piece(Colour::White),
            0b10 => Square::Piece(Colour::Black),
            _ => Square::Empty,
        }
    }

    fn encode(&self) -> usize {
        match self {
            Square::Piece(colour) => match colour {
                Colour::White => 0b01,
                Colour::Black => 0b10,
            },
            Square::Empty => 0b00,
        }
    }

    fn flip(&self) -> Self {
        match self {
            Square::Piece(colour) => Square::Piece(match colour {
                Colour::White => Colour::Black,
                Colour::Black => Colour::White,
            }),
            Square::Empty => Square::Empty,
        }
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (mut w, mut b) = ('♙', '♟');
        if is_dark_mode() {
            (w, b) = (b, w);
        }
        
        write!(f, "{}", match self {
            Square::Piece(colour) => match colour {
                Colour::White => w,
                Colour::Black => b,
            },
            Square::Empty => ' ',
        })
    }
}

fn is_dark_mode() -> bool {
    return true;
    if let Ok(theme) = env::var("COLORTERM") {
        if theme.contains("truecolor") {
            if let Ok(appearance) = env::var("TERM_THEME") {
                return appearance.to_lowercase().contains("dark");
            }
        }
    }
    false
}

#[derive(Clone, Copy, Debug)]
pub enum Colour {
    White,
    Black,
}

impl FromStr for Colour {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "W" => Ok(Self::White),
            "B" => Ok(Self::Black),
            _ => Err(anyhow!("invalid colour character"))
        }
    }
}

// POSITION

#[derive(Clone, Copy, Debug)]
struct Position {
    file: File,
    rank: Rank,
}

impl Position {
    fn new(file: File, rank: Rank) -> Self {
        Self { file, rank }
    }

    fn parse(v: impl AsRef<str>) -> Self {
        let mut v = v.as_ref().chars();
        let file = File::parse(v.next().unwrap());
        let rank = Rank::parse(v.next().unwrap());
        Self { file, rank }
    }

    const fn flip(&self) -> Self {
        Self {
            file: self.file.flip(),
            rank: self.rank.flip(),
        }
    }
}

impl From<(char, u32)> for Position {
    fn from((x, y): (char, u32)) -> Self {
        Self::new(File::parse(x), Rank::new(y))
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file, self.rank)
    }
}

#[derive(Clone, Copy, Debug)]
struct File(u32);

impl File {
    const fn new(v: u32) -> Self {
        Self(v - 1)
    }

    const fn parse(v: char) -> Self {
        Self(v.to_ascii_uppercase() as u32 - 'A' as u32)
    }

    const fn flip(&self) -> Self {
        Self(7 - self.0)
    }
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", char::from_u32('A' as u32 + self.0).unwrap())
    }
}

#[derive(Clone, Copy, Debug)]
struct Rank(u32);

impl Rank {
    const fn new(v: u32) -> Self {
        Self(v - 1)
    }

    const fn parse(v: char) -> Self {
        Self::new(v.to_digit(10).unwrap())
    }

    const fn flip(&self) -> Self {
        Self(7 - self.0)
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0 + 1)
    }
}