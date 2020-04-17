use crate::types::*;

//#################################################################################################
//
//                                     MACROS
//
//#################################################################################################

/*
 * Below are some macros to shift a bitboard in a given direction while avoid wrapping from one
 * side to the other.
 */

macro_rules! north_east {
    ($x: ident) => { ($x & 0xFEFEFEFEFEFEFEFE).wrapping_shl(7) }
}

macro_rules! north {
    ($x: ident) => { $x.wrapping_shl(8) }
}

macro_rules! north_west {
    ($x: ident) => { ($x & 0x7F7F7F7F7F7F7F7F).wrapping_shl(9) }
}

macro_rules! west {
    ($x: ident) => { ($x & 0x7F7F7F7F7F7F7F7F).wrapping_shl(1) }
}

macro_rules! east {
    ($x: ident) => { ($x & 0xFEFEFEFEFEFEFEFE).wrapping_shr(1) }
}

macro_rules! south_west {
    ($x: ident) => { ($x & 0x7F7F7F7F7F7F7F7F).wrapping_shr(7) }
}

macro_rules! south {
    ($x: ident) => { $x.wrapping_shr(8) }
}

macro_rules! south_east {
    ($x: ident) => { ($x & 0xFEFEFEFEFEFEFEFE).wrapping_shr(9) }
}

//#################################################################################################
//
//                                    OTHELLO TYPE
//
//#################################################################################################

/*
 * An Othello board only needs two BitBoards. First BitBoard is White's and second is Black's.
 */
#[derive(Clone, Copy)]
pub struct Othello(BitBoard, BitBoard);

impl Othello {
    /*
     * Creates a new Othello board in the starting position.
     */
    pub fn new() -> Othello {
        Othello(0x0000001008000000, 0x0000000810000000)
    }

    /*
     * Creates a new Othello with the given BitBoards.
     */
    #[inline(always)]
    fn create(white: BitBoard, black: BitBoard) -> Othello {
        Othello(white, black)
    }

    /*
     * Returns the BitBoard associated with the color given in argument.
     */
    #[inline(always)]
    pub fn get_bitboard(&self, color: Color) -> BitBoard {
        match color {
            Color::White => self.0,
            Color::Black => self.1,
        }
    }

    /*
     * Generates all legal moves for the given color and returns the result as a
     * BitBoard.
     */
    pub fn gen_moves(&self, playing: Color) -> BitBoard {
        let own: BitBoard = self.get_bitboard(playing);
        let opp: BitBoard = self.get_bitboard(playing.invert());

        //let mut w: BitBoard;
        let mut t: BitBoard;
        let mut moves: BitBoard = 0;

        macro_rules! search_in_direction {
            ($dir: ident) => {
                t = opp & $dir!(own);
                t |= opp & $dir!(t);
                t |= opp & $dir!(t);
                t |= opp & $dir!(t);
                t |= opp & $dir!(t);
                t |= opp & $dir!(t);
                moves |= $dir!(t);
            }
        }

        search_in_direction!(north_east);
        search_in_direction!(north);
        search_in_direction!(north_west);
        search_in_direction!(west);
        search_in_direction!(east);
        search_in_direction!(south_west);
        search_in_direction!(south);
        search_in_direction!(south_east);

        moves &= !(own | opp);

        moves
    }

    /*
     * Makes the given move on the board and returns the new board.
     */
    pub fn make_move(&self, playing: Color, mv: BitBoard) -> Othello {
        let mut own: BitBoard = self.get_bitboard(playing);
        let mut opp: BitBoard = self.get_bitboard(playing.invert());

        //let mut w: BitBoard;
        let mut c: BitBoard;
        let mut t: BitBoard;

        own |= mv;

        macro_rules! change_in_direction {
            ($dir: ident) => {
                c = opp & $dir!(mv);
                if c != 0 {
                    t = c;
                    loop {
                        c = opp & $dir!(c);
                        if c == 0 { break; }
                        t |= c;
                    }
                    if $dir!(t) & own != 0 {
                        opp ^= t;
                        own ^= t;
                    }
                }
            }
        }

        change_in_direction!(north_east);
        change_in_direction!(north);
        change_in_direction!(north_west);
        change_in_direction!(west);
        change_in_direction!(east);
        change_in_direction!(south_west);
        change_in_direction!(south);
        change_in_direction!(south_east);

        if playing == Color::White {
            Self::create(own, opp)
        } else {
            Self::create(opp, own)
        }
    }

    /*
     * Returns the state of the square at (x, y), where x and y are in 0..8.
     */
    pub fn get_square(&self, x: u8, y: u8) -> Square {
        if self.get_bitboard(Color::White).contains(x, y) {
            Square::White
        } else if self.get_bitboard(Color::Black).contains(x, y) {
            Square::Black
        } else {
            Square::Empty
        }
    }

    /*
     * Returns the score associated with the given board, that is, a simple count
     * of how many disks each player has.
     */
    pub fn score(&self) -> (u8, u8) {
        return (self.get_bitboard(Color::Black).pop_cnt(),
                self.get_bitboard(Color::White).pop_cnt());
    }
}

//#################################################################################################
//
//                                  PERFT CORRECTNESS TEST
//
//#################################################################################################

/*
 * depth   leaf nodes count
 *     1                  4
 *     2                 12
 *     3                 56
 *     4                244
 *     5               1396
 *     6               8200
 *     7              55092
 *     8             390216
 *     9            3005288
 *    10           24571284
 *    11          212258800
 *    12         1939886636
 */

/*
 * Test functions that carries a perft type test of depth DEPTH (changeable).
 */
#[test]
fn test_othello() {
    let depth: usize = 10;

    let perft_table = vec![
        1, 4, 12, 56, 244, 1396, 8200,
        55092, 390216, 3005288, 24571284,
        212258800, 1939886636,
    ];

    assert!(depth < perft_table.len(), "Depth must be at most {}", perft_table.len() - 1);

    /*
     * The perft function in itself, that counts the number of leaf nodes at depth 9.
     */
    fn perft(oth: Othello, color: Color, depth: usize) -> u64 {
        if depth == 0 { return 1; }

        let mut res: u64 = 0;
        let mut moves: BitBoard = oth.gen_moves(color);

        if moves == 0 {
            moves = oth.gen_moves(color.invert());
            if moves == 0 { return 1; }
            return perft(oth, color.invert(), depth-1);
        }

        while moves != 0 {
            res += perft(oth.make_move(color, moves.pop_lsb()), color.invert(), depth-1)
        }

        return res;
    }

    let res: u64 = perft(Othello::new(), Color::Black, depth);

    assert_eq!(res, perft_table[depth], "Got an incorrect perft value for a depth of {}", depth);
}
