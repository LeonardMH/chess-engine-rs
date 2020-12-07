use serde::{Serialize, Deserialize, Serializer};
use serde_json;

use super::board::{Coordinate};
use serde::ser::SerializeStruct;
use crate::board::coordinate::CoordinateAlgebraic;

#[derive(Serialize, Deserialize)]
pub enum Rank {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

pub enum Position {
    /// Piece was captured by the opponent
    Captured,

    /// Piece is not on the board, but also not captured by the opponent, for example when a pawn
    /// is promoted
    OtherwiseOffBoard,

    /// Piece is still on the board, must specify where
    Board(Coordinate),
}

impl Serialize for Position {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        let mut state = serializer.serialize_struct("Position", 1)?;

        let serialization = match &self {
            Position::Captured => { String::from("captured") },
            Position::OtherwiseOffBoard => { String::from("off") },
            Position::Board(coord) => {
                let algebraic = CoordinateAlgebraic::from(*coord);
                format!("{}{}", algebraic.file(), algebraic.rank())
            }
        };

        state.serialize_field("position", &serialization);
        state.end()
    }
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

impl Serialize for Piece {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        let mut state = serializer.serialize_struct("Piece", 2)?;

        state.serialize_field("rank", &self.rank);
        state.serialize_field("position", &self.position);

        state.end()
    }
}