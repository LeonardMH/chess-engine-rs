use super::board::{Coordinate};

pub enum Rank {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

pub enum Position {
    Captured,
    Board(Coordinate),
}

pub struct Piece {
    rank: Rank,
    position: Position,
}

impl Piece {
    pub fn new(rank: Rank, position: Position) -> Piece {
        Piece { rank, position }
    }
}