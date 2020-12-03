pub mod coordinate;

pub use crate::piece::Piece;
pub use coordinate::Coordinate;
use crate::board::coordinate::CoordinateLinear;

#[derive(Debug, PartialEq)]
enum SquareColor {
    Dark,
    Light,
}

impl SquareColor {
    fn color_for_coordinate(coordinate: Coordinate) -> SquareColor {
        let bm_dark: u64 = 0xAA55AA55AA55AA55;
        let square_index = CoordinateLinear::from(coordinate).index();

        if ((bm_dark >> square_index) & 1) != 0 {
            SquareColor::Dark
        } else {
            SquareColor::Light
        }
    }
}

pub struct Square {
    piece: Option<Piece>,
    color: SquareColor,
    coordinate: Coordinate,
}

impl Square {
    fn new(coordinate: Coordinate, piece: Option<Piece>) -> Square {
        Square {
            piece,
            color: SquareColor::color_for_coordinate(coordinate),
            coordinate,
        }
    }
}

pub struct Board {
    squares: [Square; 64],
}

#[cfg(test)]
mod tests {
    use crate::board::{Coordinate, SquareColor};
    use crate::board::SquareColor::{Dark, Light};
    use crate::board::coordinate::CoordinateLinear;

    #[test]
    fn test_color_determination() {
        let color_by_index_lut = [
            Dark, Light, Dark, Light, Dark, Light, Dark, Light,
            Light, Dark, Light, Dark, Light, Dark, Light, Dark,
            Dark, Light, Dark, Light, Dark, Light, Dark, Light,
            Light, Dark, Light, Dark, Light, Dark, Light, Dark,
            Dark, Light, Dark, Light, Dark, Light, Dark, Light,
            Light, Dark, Light, Dark, Light, Dark, Light, Dark,
            Dark, Light, Dark, Light, Dark, Light, Dark, Light,
            Light, Dark, Light, Dark, Light, Dark, Light, Dark,
        ];

        for (index, expect) in color_by_index_lut.iter().enumerate() {
            // we can unwrap as this is test code and we will only test valid values
            let coord = Coordinate::from(CoordinateLinear::new(index as u8).unwrap());
            assert_eq!(SquareColor::color_for_coordinate(coord), *expect);
        }
    }
}
