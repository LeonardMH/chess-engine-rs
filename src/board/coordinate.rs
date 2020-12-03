/// Represents the coordinate system on the board
///
/// Three possible coordinate systems are currently allowed for:
/// - XY where the a1 square is (0, 0) and the h8 square is (7, 7)
/// - Algebraic which is the standard a1 - h8 notation
/// - Linear which tends to be convenient for computers, a1 is 0, h1 is 7, h8 is 63
///
/// For optimization reasons, I'm only storing the XY coordinates in the struct itself,
/// the others can be derived.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CoordinateXY {
    x: u8,
    y: u8,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CoordinateAlgebraic {
    file: char,
    rank: char,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CoordinateLinear {
    index: u8,
}

/// NOTE: This is what sets the default coordinate system
pub type Coordinate = CoordinateXY;

#[derive(Debug, Clone)]
pub enum CoordinateError {
    /// Attempting to construct or access a coordinate that is outside of the allowed board area
    OutOfBounds,

    /// Poorly specified input when attempting to instantiate a coordinate
    BadFormat,
}

type Result<T> = std::result::Result<T, CoordinateError>;

impl CoordinateXY {
    pub fn new(x: u8, y: u8) -> Result<CoordinateXY> {
        Ok(CoordinateXY { x, y })
    }

    pub fn x(&self) -> u8 { self.x }
    pub fn y(&self) -> u8 { self.y }
}

impl CoordinateLinear {
    pub fn new(index: u8) -> Result<CoordinateLinear> {
        Ok(CoordinateLinear { index })
    }

    pub fn index(&self) -> u8 { self.index }
}

impl CoordinateAlgebraic {
    pub fn new(file: char, rank: char) -> Result<CoordinateAlgebraic> {
        Ok(CoordinateAlgebraic { file, rank })
    }

    pub fn file(&self) -> char { self.file }
    pub fn rank(&self) -> char { self.rank }
}

/// From pure Coordinate type to other subtypes
impl From<CoordinateXY> for CoordinateLinear {
    fn from(coord: CoordinateXY) -> CoordinateLinear {
        // converting from x-y to linear coordinates is simple: x + 8y
        // we can unwrap the result as any valid Coordinate type can be converted to another
        CoordinateLinear::new(coord.x + 8 * coord.y).unwrap()
    }
}

impl From<CoordinateXY> for CoordinateAlgebraic {
    fn from(coord: CoordinateXY) -> CoordinateAlgebraic {
        // converting from x-y to algebraic coordinates is also simple, we can just use
        // two lookup tables. We could also do this using some ASCII math, but we don't really
        // gain much by going that route on this implementation
        let file_lut = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        let rank_lut = ['1', '2', '3', '4', '5', '6', '7', '8'];

        let file = file_lut[coord.x as usize];
        let rank = rank_lut[coord.y as usize];

        // we can unwrap the result as any valid Coordinate type can be converted to another
        CoordinateAlgebraic::new(file, rank).unwrap()
    }
}

/// From subtypes into pure Coordinate type
impl From<CoordinateLinear> for CoordinateXY {
    fn from(coord: CoordinateLinear) -> CoordinateXY {
        // this conversion is dependent on the board size/shape, but it's just basic div/mod math
        let y = coord.index / 8;
        let x = coord.index % 8;

        // we can unwrap the result as any valid Coordinate type can be converted to another
        CoordinateXY::new(x, y).unwrap()
    }
}

impl From<CoordinateAlgebraic> for CoordinateXY {
    fn from(coord: CoordinateAlgebraic) -> CoordinateXY {
        // probably the most complex conversion (from an "I barely know what I'm doing
        // in Rust" standpoint at least). As this is a low level function which I suspect
        // might get a lot of use, I am going to assume the inputs to this function are valid
        // CoordinateAlgebraic characters
        //
        // we need to map ('a'-'h' == 0-7) and ('1'-'8' == 0-7)
        //                  97-104               49-56
        //
        // the x coordinate corresponds to the file (letter) of the algebraic coordinate
        // the y coordinate corresponds to the rank (number) of the algebraic coordinate
        let x = coord.file as u8 - 97;
        let y = coord.rank as u8 - 49;

        // we can unwrap the result as any valid Coordinate type can be converted to another
        CoordinateXY::new(x, y).unwrap()
    }
}

/// Conversion between subtypes
impl From<CoordinateLinear> for CoordinateAlgebraic {
    fn from(coord: CoordinateLinear) -> CoordinateAlgebraic {
        // go from linear to xy and then xy to alg
        let xy = CoordinateXY::from(coord);
        xy.into()
    }
}

impl From<CoordinateAlgebraic> for CoordinateLinear {
    fn from(coord: CoordinateAlgebraic) -> CoordinateLinear {
        // go from algebraic to xy then to linear
        let xy = CoordinateXY::from(coord);
        xy.into()
    }
}

#[cfg(test)]
mod tests {
    use crate::board::coordinate::{CoordinateAlgebraic, CoordinateLinear, CoordinateXY};

    static TEST_SET: [((u8, u8), (char, char), u8); 24] = [
        // move along the 1 rank
        ((0, 0), ('a', '1'), 0),
        ((1, 0), ('b', '1'), 1),
        ((2, 0), ('c', '1'), 2),
        ((3, 0), ('d', '1'), 3),
        ((4, 0), ('e', '1'), 4),
        ((5, 0), ('f', '1'), 5),
        ((6, 0), ('g', '1'), 6),
        ((7, 0), ('h', '1'), 7),

        // move along the a file
        ((0, 0), ('a', '1'), 0),
        ((0, 1), ('a', '2'), 8),
        ((0, 2), ('a', '3'), 16),
        ((0, 3), ('a', '4'), 24),
        ((0, 4), ('a', '5'), 32),
        ((0, 5), ('a', '6'), 40),
        ((0, 6), ('a', '7'), 48),
        ((0, 7), ('a', '8'), 56),

        // move along the a1-h8 diagonal
        ((0, 0), ('a', '1'), 0),
        ((1, 1), ('b', '2'), 9),
        ((2, 2), ('c', '3'), 18),
        ((3, 3), ('d', '4'), 27),
        ((4, 4), ('e', '5'), 36),
        ((5, 5), ('f', '6'), 45),
        ((6, 6), ('g', '7'), 54),
        ((7, 7), ('h', '8'), 63),
    ];

    #[test]
    fn test_coordinate_conv_xy_to_linear() {
        for ((x, y), _, linear) in TEST_SET.iter() {
            let coord = CoordinateXY::new(*x, *y).unwrap();
            let result= CoordinateLinear::from(coord);
            let expect = CoordinateLinear::new(*linear).unwrap();

            assert_eq!(result, expect);
        }
    }

    #[test]
    fn test_coordinate_conv_linear_to_xy() {
        for ((x, y), _, linear) in TEST_SET.iter() {
            let coord = CoordinateLinear::new(*linear).unwrap();
            let result = CoordinateXY::from(coord);
            let expect = CoordinateXY::new(*x, *y).unwrap();

            assert_eq!(result, expect);
        }
    }

    #[test]
    fn test_coordinate_conv_xy_to_algebraic() {
        for ((x, y), algebraic, _) in TEST_SET.iter() {
            let coord = CoordinateXY::new(*x, *y).unwrap();
            let result = CoordinateAlgebraic::from(coord);
            let expect = CoordinateAlgebraic::new(algebraic.0, algebraic.1).unwrap();

            assert_eq!(result, expect);
        }
    }

    #[test]
    fn test_coordinate_conv_algebraic_to_xy() {
        for ((x, y), algebraic, _) in TEST_SET.iter() {
            let coord = CoordinateAlgebraic::new(algebraic.0, algebraic.1).unwrap();
            let result = CoordinateXY::from(coord);
            let expect = CoordinateXY::new(*x, *y).unwrap();

            assert_eq!(result, expect);
        }
    }

    #[test]
    fn test_all_way_commutative_conversion() {
        // convert each test coordinate from XY to linear to algebraic and back to XY, testing
        // the full conversion range
        for ((x, y), algebraic, linear) in TEST_SET.iter() {
            let xy_expect = CoordinateXY::new(*x, *y).unwrap();
            let alg_expect = CoordinateAlgebraic::new(algebraic.0, algebraic.1).unwrap();
            let lin_expect = CoordinateLinear::new(*linear).unwrap();

            // from x-y to linear
            let result = CoordinateLinear::from(xy_expect);
            assert_eq!(result, lin_expect);

            // from linear to algebraic
            let result = CoordinateAlgebraic::from(lin_expect);
            assert_eq!(result, alg_expect);

            // from algebraic to xy
            let result = CoordinateXY::from(alg_expect);
            assert_eq!(result, xy_expect);
        }
    }
}
