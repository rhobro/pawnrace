use anyhow::anyhow;
use std::{env, fmt::Display, ops::Rem, rc::Rc, str::FromStr};

use crate::IO;

pub fn test(mut io: IO) {
    let mut b = Board::new();
    b.set(&('g', 6).into(), Square::Piece(Colour::White));
    println!("{b}");

    for m in b.moves() {
        println!("{} to {}", m.piece.pos, m.to);
    }
}

// BOARD

// Storing chessboard board as bits
// Assuming white starts from bottom
// 2 bits per square:
//  - white: 01
//  - black: 10
//  - empty: 00, 11
// Using this config so that reversing bits reverses colours
#[derive(Clone, Copy)]
pub struct Board {
    raw: u128,
    passantable_pos: Option<Position>,
}

impl Board {
    pub const fn new() -> Self {
        Self {
            // full board
            raw: 0x0000AAAA000000000000000055550000,
            passantable_pos: None,
        }
    }

    pub fn after(&self, m: &Move) -> Self {
        let mut b = *self;

        // delete old
        b.set(&m.piece.pos, Square::Empty);
        // set new
        b.set(&m.to, Square::Piece(Colour::White));
        // delete en passant
        if m.passant {
            b.set(
                m.piece.board.passantable_pos.as_ref().unwrap(),
                Square::Empty,
            );
        }
        // register potential passant
        if m.to.rank.n() - m.piece.pos.rank.n() == 2 {
            b.passantable_pos = Some(m.to);
        }

        b
    }

    // return only white pawns
    pub fn pieces(self) -> PieceIterator {
        PieceIterator::new(Rc::new(self))
    }

    pub fn moves(&self) -> impl Iterator<Item = Move> {
        self.pieces().flat_map(|p| p.moves())
    }

    pub fn flip(&self) -> Self {
        Self {
            raw: self.raw.reverse_bits(),
            passantable_pos: self.passantable_pos.map(|pos| pos.flip()),
        }
    }

    pub fn at(&self, pos: &Position) -> Square {
        let bit_pos = Self::bits_at(pos);
        let mask: u128 = 0b11 << bit_pos;
        Square::decode((self.raw & mask) >> bit_pos)
    }

    fn set(&mut self, pos: &Position, s: Square) {
        let bit_pos = Self::bits_at(pos);

        // clear to 00
        let mut mask: u128 = (0b11 << bit_pos) ^ u128::MAX;
        self.raw &= mask;
        // set to encoded
        mask = s.encode() << bit_pos;
        self.raw |= mask;
    }

    fn bits_at(pos: &Position) -> i32 {
        2 * (8 * pos.rank.0 + pos.file.0)
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "    -----------------")?;

        for rank in (1..9).rev() {
            write!(f, " {rank} |")?;

            for file in 1..9 {
                let pos: Position = (file, rank).into();
                write!(f, " {}", self.at(&pos))?;
            }

            writeln!(f, " |")?;
        }

        // end
        writeln!(f, "    -----------------")?;
        writeln!(f, "     A B C D E F G H")
    }
}

#[derive(Clone, Copy)]
pub enum Square {
    Piece(Colour),
    Empty,
}

impl Square {
    fn decode(v: u128) -> Self {
        match v {
            0b01 => Square::Piece(Colour::White),
            0b10 => Square::Piece(Colour::Black),
            _ => Square::Empty,
        }
    }

    fn encode(&self) -> u128 {
        match self {
            Square::Piece(colour) => match colour {
                Colour::White => 0b01,
                Colour::Black => 0b10,
            },
            Square::Empty => 0b00,
        }
    }

    pub fn flip(&self) -> Self {
        match self {
            Square::Piece(colour) => Square::Piece(match colour {
                Colour::White => Colour::Black,
                Colour::Black => Colour::White,
            }),
            Square::Empty => Square::Empty,
        }
    }

    pub fn is_white(&self) -> bool {
        matches!(self, Square::Piece(Colour::White))
    }

    pub fn is_black(&self) -> bool {
        matches!(self, Square::Piece(Colour::Black))
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Square::Empty)
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (mut w, mut b) = ('♙', '♟');
        if is_dark_mode() {
            (w, b) = (b, w);
        }

        write!(
            f,
            "{}",
            match self {
                Square::Piece(colour) => match colour {
                    Colour::White => w,
                    Colour::Black => b,
                },
                Square::Empty => ' ',
            }
        )
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
            _ => Err(anyhow!("invalid colour character")),
        }
    }
}

// PIECE and MOVE

pub struct Piece {
    pos: Position,
    board: Rc<Board>,
}

impl Piece {
    pub fn moves(self) -> MovesIterator {
        MovesIterator::new(Rc::new(self))
    }
}

pub struct Move {
    to: Position,
    passant: bool,
    piece: Rc<Piece>,
}

// POSITION

#[derive(Clone, Copy, PartialEq)]
pub struct Position {
    file: File,
    rank: Rank,
}

impl Position {
    pub const fn flip(&self) -> Self {
        Self {
            file: self.file.flip(),
            rank: self.rank.flip(),
        }
    }

    // moves L --> R, B --> T
    pub fn incr(&self) -> Self {
        Self {
            file: self.file.incr(),
            rank: if self.file.is_end() {
                self.rank.incr()
            } else {
                self.rank
            },
        }
    }

    pub fn left(&self) -> Option<Self> {
        if self.file.is_start() {
            return None;
        }

        Some(Self {
            file: self.file.decr(),
            rank: self.rank,
        })
    }

    pub fn right(&self) -> Option<Self> {
        if self.file.is_end() {
            return None;
        }

        Some(Self {
            file: self.file.incr(),
            rank: self.rank,
        })
    }

    pub fn front(&self) -> Option<Self> {
        if self.rank.is_end() {
            return None;
        }

        Some(Self {
            file: self.file,
            rank: self.rank.incr(),
        })
    }

    pub fn diag_l(&self) -> Option<Self> {
        self.front().and_then(|pos| pos.left())
    }

    pub fn diag_r(&self) -> Option<Self> {
        self.front().and_then(|pos| pos.right())
    }

    pub const fn is_start(&self) -> bool {
        self.file.is_start() && self.rank.is_start()
    }

    pub const fn is_end(&self) -> bool {
        self.file.is_end() && self.rank.is_end()
    }
}

impl From<(char, i32)> for Position {
    fn from((x, y): (char, i32)) -> Self {
        Self {
            file: File::parse(x),
            rank: Rank::new(y),
        }
    }
}

impl From<(i32, i32)> for Position {
    fn from((x, y): (i32, i32)) -> Self {
        Self {
            file: File::new(x),
            rank: Rank::new(y),
        }
    }
}

impl From<(File, Rank)> for Position {
    fn from((file, rank): (File, Rank)) -> Self {
        Self { file, rank }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file, self.rank)
    }
}

#[derive(Clone, Copy, PartialEq)]
struct File(i32);

impl File {
    pub fn new(v: i32) -> Self {
        Self((v - 1).wrapping_rem(8))
    }

    pub fn parse(v: char) -> Self {
        Self(v.to_ascii_uppercase() as i32 - 'A' as i32)
    }

    pub fn n(&self) -> i32 {
        self.0 + 1
    }

    pub fn v(&self) -> char {
        char::from_u32('A' as u32 + self.0 as u32).unwrap()
    }

    pub const fn flip(&self) -> Self {
        Self(7 - self.0)
    }

    pub const fn is_start(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_end(&self) -> bool {
        self.0 == 7
    }

    pub fn incr(&self) -> Self {
        Self::new(self.n() + 1)
    }

    pub fn decr(&self) -> Self {
        Self::new(self.n() - 1)
    }
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.v())
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Rank(i32);

impl Rank {
    pub fn new(v: i32) -> Self {
        Self((v - 1).wrapping_rem(8))
    }

    pub fn parse(v: char) -> Self {
        Self::new(v.to_digit(10).unwrap() as i32)
    }

    pub fn n(&self) -> i32 {
        self.0 + 1
    }

    pub const fn flip(&self) -> Self {
        Self(7 - self.0)
    }

    pub const fn is_start(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_end(&self) -> bool {
        self.0 == 7
    }

    pub fn incr(&self) -> Self {
        Self::new(self.n() + 1)
    }

    pub fn decr(&self) -> Self {
        Self::new(self.n() - 1)
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0 + 1)
    }
}

// ITERATORS

pub struct PieceIterator {
    board: Rc<Board>,
    look: Position,
    done: bool,
}

impl PieceIterator {
    fn new(board: Rc<Board>) -> Self {
        Self {
            board,
            look: (8, 8).into(),
            done: false,
        }
    }
}

impl Iterator for PieceIterator {
    type Item = Piece;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        while !self.done {
            self.look = self.look.incr();
            if self.look.is_end() {
                self.done = true;
            }

            if self.board.at(&self.look).is_white() {
                return Some(Piece {
                    pos: self.look,
                    board: self.board.clone(),
                });
            }
        }
        None
    }
}

pub struct MovesIterator {
    piece: Rc<Piece>,

    // have checked?
    fwd_1: bool,
    fwd_2: bool,
    // (L, R)
    diag: (bool, bool),
    passant: (bool, bool),
}

impl MovesIterator {
    fn new(piece: Rc<Piece>) -> Self {
        let passant_check = piece.board.passantable_pos.is_none();

        Self {
            piece: piece.clone(),

            fwd_1: false,
            fwd_2: piece.as_ref().pos.rank.n() != 2,
            diag: (false, false),
            passant: (passant_check, passant_check),
        }
    }
}

impl Iterator for MovesIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        // one forward
        if !self.fwd_1 {
            self.fwd_1 = true;

            // empty ahead
            if let Some(pos) = self
                .piece
                .pos
                .front()
                .filter(|pos| self.piece.board.at(pos).is_empty())
            {
                return Some(Move {
                    to: pos,
                    passant: false,
                    piece: self.piece.clone(),
                });
            } else {
                // disable two forward
                self.fwd_2 = true;
            }
        }

        // two forward
        if !self.fwd_2 {
            self.fwd_2 = true;

            // empty ahead
            if let Some(pos) = self
                .piece
                .pos
                .front()
                .and_then(|pos| pos.front())
                .filter(|pos| self.piece.board.at(pos).is_empty())
            {
                return Some(Move {
                    to: pos,
                    passant: false,
                    piece: self.piece.clone(),
                });
            }
        }

        // diag left
        if !self.diag.0 {
            self.diag.0 = true;

            // black there
            if let Some(pos) = self
                .piece
                .pos
                .diag_l()
                .filter(|pos| self.piece.board.at(pos).is_black())
            {
                return Some(Move {
                    to: pos,
                    passant: false,
                    piece: self.piece.clone(),
                });
            }
        }

        // diag right
        if !self.diag.1 {
            self.diag.1 = true;

            // black there
            if let Some(pos) = self
                .piece
                .pos
                .diag_r()
                .filter(|pos| self.piece.board.at(pos).is_black())
            {
                return Some(Move {
                    to: pos,
                    passant: false,
                    piece: self.piece.clone(),
                });
            }
        }

        // passant left
        if !self.passant.0 {
            self.passant.0 = true;

            // black left
            if self.piece.pos.left() == self.piece.board.passantable_pos {
                // empty there
                if let Some(pos) = self
                    .piece
                    .pos
                    .diag_l()
                    .filter(|pos| self.piece.board.at(pos).is_empty())
                {
                    return Some(Move {
                        to: pos,
                        passant: true,
                        piece: self.piece.clone(),
                    });
                }
            }
        }

        // passant left
        if !self.passant.1 {
            self.passant.1 = true;

            // black left
            if self.piece.pos.right() == self.piece.board.passantable_pos {
                // empty there
                if let Some(pos) = self
                    .piece
                    .pos
                    .diag_r()
                    .filter(|pos| self.piece.board.at(pos).is_empty())
                {
                    return Some(Move {
                        to: pos,
                        passant: true,
                        piece: self.piece.clone(),
                    });
                }
            }
        }

        None
    }
}