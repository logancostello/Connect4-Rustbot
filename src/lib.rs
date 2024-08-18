use std::path::Path;
use std::fs;
use std::time::{Duration, SystemTime};

struct Position {
    board: [i64; 2],
    turn: usize,
    moves: Vec<usize>,
    heights: [usize; 7]
}

impl Position {

    // given a column, returns if it can be played in or not
    pub fn  is_legal_move(&self, col: usize) -> bool {
        self.heights[col] < 6
    }

    // receives a column to play in (assumed to be legal) and plays it
    pub fn make_move(&mut self, col: usize) {
        let move_mask: i64 = (1 << (7 * col)) << self.heights[col];
        self.board[self.turn] |= move_mask;
        self.turn = 1 - self.turn;
        self.moves.push(col);
        self.heights[col] += 1;
    }

    // play multiple moves. used for tests
    pub fn make_moves(&mut self, cols: Vec<usize>) {
        for col in cols {
            self.make_move(col);
        }
    }

    // undo a move
    pub fn undo_move(&mut self) {
        let last_col = self.moves.pop().unwrap();
        self.heights[last_col] -= 1;
        self.turn = 1 - self.turn;

        let undo_mask: i64 = 1 << (7 * last_col + self.heights[last_col]);
        self.board[self.turn] ^= undo_mask;
    }

    // check if a move results in connect 4
    pub fn is_winning_move(&self, col: usize) -> bool {
        let b = self.board[self.turn] | (1 << (7 * col) << self.heights[col]);
        if ((b & b << 1 & b << 2 & b << 3) |
            (b & b << 7 & b << 14 & b << 21) |
            (b & b << 6 & b << 12 & b << 18) |
            (b & b << 8 & b << 16 & b << 24)) != 0 {
                return true
            } 
        false
    }

    // gets threats of opponent
    pub fn opponents_threats(&self) -> i64 {
        let open: i64 = !(self.board[0] | self.board[1] | 283691315109952 | 71776119061217280);
        let opp: i64 = self.board[1 - self.turn];

        let mut threats: i64 = 0;

        // vertical threats
        threats |= open & opp << 1 & opp << 2 & opp << 3;

        // horizontal threats
        threats |= open & opp << 7 & opp << 14 & opp << 21;
        threats |= opp >> 7 & open & opp << 7 & opp << 14;
        threats |= opp >> 7 & opp >> 14 & open & opp << 7;
        threats |= opp >> 7 & opp >> 14 & opp >> 21 & open;
        
        // positive diagonol threats
        threats |= open & opp << 8 & opp << 16 & opp << 24;
        threats |= opp >> 8 & open & opp << 8 & opp << 16;
        threats |= opp >> 16 & opp >> 8 & open & opp << 8;
        threats |= opp >> 24 & opp >> 16 & opp >> 8 & open;

        // negative diagonol threats
        threats |= open & opp << 6 & opp << 12 & opp << 18;
        threats |= opp >> 6 & open & opp << 6 & opp << 12;
        threats |= opp >> 12 & opp >> 6 & open & opp << 6;
        threats |= opp >> 18 & opp >> 12 & opp >> 6 & open;

        threats

    }
}

// takes a position, returns its score and how many positions were searched
fn score(pos: &mut Position, mut alpha: i8, mut beta: i8) -> (i8, u64) {

    // track # positions searched
    // unnecessary for scoring a position, but useful for tracking progress
    let mut total_positions: u64 = 1;

    // check if game is a tie
    if pos.moves.len() == 42 {return (0, total_positions)};

    // check for a winning move
    let move_options = [3, 2, 4, 1, 5, 0, 6];
    for mv in move_options {
        if pos.is_legal_move(mv) && pos.is_winning_move(mv) {
            return ((43 - pos.moves.len() as i8) / 2, total_positions);
        }
    }

    // beta should be <= the max possible score
    let max_possible_score: i8 = (41 - pos.moves.len() as i8) / 2;
    if beta > max_possible_score {
        beta = max_possible_score;
        if alpha >= beta {return (beta, total_positions)} // alpha beta window is empty
    }

    // search further
    for mv in move_options {
        if pos.is_legal_move(mv) {
            pos.make_move(mv);
            let (mut s, p) = score(pos, -1 * beta, -1 * alpha);
            pos.undo_move();
            s *= -1; 
            total_positions += p;
            if s > alpha { alpha = s };
            if alpha >= beta { break }
        }
    }
    (alpha, total_positions)
}

#[cfg(test)]
mod tests {
    use super::*;

    // returns start position
    fn start_position() -> Position {
        Position {
            board: [0, 0],
            turn: 0,
            moves: Vec::new(),
            heights: [0; 7]
        }
    }

    // takes a valid test position key and turns it into a position
    fn key_to_position(key: String) -> Position {
        let mut p = start_position();
        for keymove in key.chars() {
            // test keys use 1-7, i use 0-6
            p.make_move(keymove.to_digit(10).unwrap() as usize - 1);
        }
        p
    }

    // takes file path, tests score func on those position
    // returns [% correct, avg solve time, avg # of positions searched]
    fn check_progress<P: AsRef<Path>>(file_path: P) -> (f64, f64, u64) {
        // read the desired file
        let contents = match fs::read_to_string(&file_path) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("Error reading file: {}", e); 
                return (-1.0, -1.0, 1)
            },
        };

        let mut num_correct: f64 = 0.0;
        let mut total_time = Duration::new(0, 0);
        let mut total_num_positions: u64 = 0;

        let mut line_count = 0;
        //for each line, test the score func
        for line in contents.lines() {
            line_count += 1;
            let mut parts = line.split_whitespace();
            let key = parts.next().unwrap();
            let actual_score: i8 = parts.next().unwrap().parse().expect("Not a valid number");

            let mut p = key_to_position(key.to_string());
            let start = SystemTime::now();
            let (predicted_score, num_positions) = score(&mut p, -22, 22);
            let end = SystemTime::now();
            
            let duration = end.duration_since(start).expect("Time went backwards");
            if actual_score == predicted_score { 
                num_correct += 1.0;
                println!("{line_count}: {} {key}", duration.as_secs()); 
            } else {
                println!("INCORRECT {line_count}: {} {key}", duration.as_secs());
            };

            total_time += duration;
            total_num_positions += num_positions;
        }
        // return progress information
        (num_correct / 10.0, (total_time.as_micros() / 1000) as f64 / 1_000_000.0, total_num_positions/ 1000)
    }

    #[test]
    fn test_make_move_0() {
        let mut p = start_position();

        p.make_move(0);
        assert_eq!(p.board[0], 1);
        assert_eq!(p.board[1], 0);
        assert_eq!(p.heights[0], 1);
        assert_eq!(p.turn, 1);
        assert_eq!(p.moves, vec![0]);
    }

    #[test]
    fn test_make_move_1() {
        let mut p = start_position();

        p.make_move(3);
        assert_eq!(p.board[0], i64::pow(2, 21));
        assert_eq!(p.board[1], 0);
        assert_eq!(p.heights, [0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(p.turn, 1);
        assert_eq!(p.moves, vec![3]);
    }

    #[test]
    fn test_make_move_2() {
        let mut p = start_position();

        p.make_moves(vec![0, 0, 0]);
        assert_eq!(p.moves, vec![0, 0, 0]);
        assert_eq!(p.board, [5, 2]);
        assert_eq!(p.turn, 1);
        assert_eq!(p.heights, [3, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_undo_move_0() {
        let mut p = start_position();
        
        p.make_move(1);
        p.undo_move();

        assert_eq!(p.board, [0, 0]);
        assert_eq!(p.turn, 0);
    }
 
    #[test]
    fn test_undo_move_1() {
        let mut p = start_position();

        p.make_moves(vec![1, 2, 4, 6, 2, 2, 2, 1]);
        for _i in 0..8 {
            p.undo_move();
        }

        assert_eq!(p.board, [0, 0]);
        assert_eq!(p.turn, 0);
        assert_eq!(p.moves, vec![]);
        assert_eq!(p.heights, [0; 7]);
    }

    #[test]
    fn test_is_legal_move_0() {
        let mut p = start_position();
        p.make_moves(vec![3, 3, 3, 3, 3, 3, 5, 5, 5, 5, 5, 5, 1, 2, 4, 6, 1, 1, 1]);
        
        assert_eq!(p.is_legal_move(0), true);
        assert_eq!(p.is_legal_move(1), true);
        assert_eq!(p.is_legal_move(2), true);
        assert_eq!(p.is_legal_move(3), false);
        assert_eq!(p.is_legal_move(4), true);
        assert_eq!(p.is_legal_move(5), false);
        assert_eq!(p.is_legal_move(6), true);
    }

    #[test]
    fn test_is_winning_move_0() { // horizontal
        let mut p = start_position();
        p.make_moves(vec![3, 3, 2, 2, 4, 4]);
        assert_eq!(p.is_winning_move(5), true);
    }

    #[test]
    fn test_is_winning_move_1() { // vertical
        let mut p = start_position();
        p.make_moves(vec![3, 2, 3, 2, 3, 2, 0]);
        assert_eq!(p.is_winning_move(2), true);
    }

    #[test]
    fn test_is_winning_move_2() { // positive diagonol
        let mut p = start_position();
        p.make_moves(vec![0, 1, 1, 2, 2, 3, 2, 3, 3, 4]);
        assert_eq!(p.is_winning_move(3), true);
    }

    #[test]
    fn test_is_winning_move_3() { // negative diagonol
        let mut p = start_position();
        p.make_moves(vec![0, 6, 5, 5, 4, 4, 3, 4, 3, 3, 2]);
        assert_eq!(p.is_winning_move(3), true);
    }

    #[test]
    fn test_is_winning_move_4() { // vertical wrapping 
        let mut p = start_position();
        p.make_moves(vec![0, 0, 0, 0, 3, 0, 3, 0, 3]);
        assert_eq!(p.is_winning_move(1), false);
    }

    #[test]
    fn test_key_to_position_0() {
        let p = key_to_position(String::from("111"));

        assert_eq!(p.board, [5, 2]);
        assert_eq!(p.turn, 1);
        assert_eq!(p.moves, vec![0, 0, 0]);
        assert_eq!(p.heights, [3, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_score_0() { // will be a tie in 1 move
        let mut pos = key_to_position(String::from("11111122222234333334444455555567676776767"));
        let (s, _p) = score(&mut pos, -22, 22);
        assert_eq!(s, 0);
    }

    #[test]
    fn test_score_1() { // will be a tie in 5 moves
        let mut pos = key_to_position(String::from("1111112222223433333444445555556767677"));
        let (s, _p) = score(&mut pos, -22, 22);
        assert_eq!(s, 0);
    }

    #[test]
    fn test_score_2() { // player 1 can win in 2 moves
        let mut pos = key_to_position(String::from("1111112222223433333444445555556767"));
        let (s, _p) = score(&mut pos, -22, 22);
        assert_eq!(s, 3);
    }

    #[test]
    fn test_score_3() { // player 1 loses in 3 moves
        let mut pos = key_to_position(String::from("1111112222223433333444445555556766"));
        let (s, _p) = score(&mut pos, -22, 22);
        assert_eq!(s, -2);
    }

    #[test]
    fn test_score_4() { // player 2 can win in 4 moves
        let mut pos = key_to_position(String::from("111111222222343333344444555555676"));
        let (s, _p) = score(&mut pos, -22, 22);
        assert_eq!(s, 2);
    }

    #[test]
    fn test_threats_0() { // horizontal
        let mut pos = start_position();
        pos.make_moves(vec![1, 1, 2, 2, 3, 3]);

        assert_eq!(pos.opponents_threats(), 2 + 2_i64.pow(29));
    }

    #[test]
    fn test_threats_1() { // vertical
        let mut pos = start_position();
        pos.make_moves(vec![0, 1, 0, 1, 0, 1]);

        assert_eq!(pos.opponents_threats(), 2_i64.pow(10));

        pos.make_moves(vec![1, 0, 5, 6, 5, 6]);
        assert_eq!(pos.opponents_threats(), 0);
    }

    #[test]
    fn test_threats_2() { // positive diagonol
        let mut pos = start_position();
        pos.make_moves(vec![1, 1, 2, 3, 2, 2, 3, 3, 6, 3]);

        assert_eq!(pos.opponents_threats(), 1 + 2_i64.pow(32));
    } 

    #[test]
    fn test_threats_3() { // negative diagonol
        let mut pos = start_position();
        pos.make_moves(vec![5, 5, 4, 3, 4, 4, 3, 3, 1, 3]);

        assert_eq!(pos.opponents_threats(), 2_i64.pow(42) + 2_i64.pow(18));
    } 

    // #[test]
    // fn test_progress_check() { // used to check efficiency progress, will not pass
    //     let result = check_progress("test_files/End-Easy.txt");
    //     assert_eq!(result, (0.0, 0.0, 0));
    // }
}
