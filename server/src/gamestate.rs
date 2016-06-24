
pub const BOARD_SIZE: usize = 19;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TileState {
    Empty,
    White,
    Black
}

pub enum Move {
    Stone((usize, usize)),
    Pass
}

pub struct GameState {
    pub board: [[TileState; BOARD_SIZE]; BOARD_SIZE],
}

impl GameState {
    pub fn is_full(&self) -> bool {
        for row in self.board.iter() {
            for pos in row.iter() {
                match *pos {
                    TileState::Empty => return false,
                    _ => (),
                }
            }
        }
        return true;
    }
    pub fn get_tile(&self, pos: (usize, usize)) -> TileState{
        let (x, y) = pos;
        return self.board[x][y];
    }
}

impl ToString for GameState {
    fn to_string(&self) -> String{
        let mut result = String::from("");
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                match self.board[x][y] {
                    TileState::Empty => result.push_str("."),
                    TileState::White => result.push_str("W"),
                    TileState::Black => result.push_str("B"),
                }
            }
            result = result + "\n";
        }
        return result;
    }
}
