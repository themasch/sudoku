use std::ops::{BitXorAssign, Shr};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Numbers([u8; 81]);

impl From<[u8; 81]> for Numbers {
    fn from(value: [u8; 81]) -> Self {
        Numbers(value)
    }
}

impl Numbers {
    fn empty() -> Self {
        Numbers([0u8; 81])
    }

    fn cells(&self) -> Cells {
        Cells {
            current_index: 0,
            values: &self.0,
        }
    }

    fn set(&mut self, row: usize, col: usize, value: u8) {
        assert!((1..=9).contains(&row));
        assert!((1..=9).contains(&col));
        let index = ((row - 1) * 9) + (col - 1);
        self.0[index] = value;
    }

    fn get(&self, row: usize, col: usize) -> u8 {
        assert!((1..=9).contains(&row));
        assert!((1..=9).contains(&col));
        let index = ((row - 1) * 9) + (col - 1);

        self.0[index]
    }

    fn get_by_offset(&self, offset: usize) -> u8 {
        assert!(offset < 81);

        self.0[offset]
    }

    fn get_row(&self, row: usize) -> [u8; 9] {
        assert!((1..=9).contains(&row));
        let row_start = (row - 1) * 9;
        let row_end = row_start + 9;
        self.0[row_start..row_end].try_into().unwrap()
    }

    fn get_col(&self, col: usize) -> [u8; 9] {
        assert!((1..=9).contains(&col));
        let mut buffer = [0u8; 9];
        let offset = col - 1;
        for row in 0..9 {
            buffer[row] = self.0[(row * 9) + offset];
        }

        buffer
    }

    /// this is a bit harder than the above
    /// a box is one of the 9 3x3 squares in a sudoku grid,
    /// so we need to find nine number of three rows.
    /// find the index of the top-left cell in that box, copy the next 3 bytes,
    /// then add 9 to that index, copy the next three, and again.
    fn get_box(&self, index: usize) -> [u8; 9] {
        let cell_index = match index {
            1 => 0,
            2 => 3,
            3 => 6,
            4 => 27,
            5 => 30,
            6 => 33,
            7 => 54,
            8 => 57,
            9 => 60,
            _ => panic!("not a valid box index"),
        };

        let mut buffer = [0u8; 9];
        for row in 0..3 {
            let start = cell_index + (9 * row);
            for col in 0..3 {
                buffer[row * 3 + col] = self.0[start + col];
            }
        }

        buffer
    }
}

pub struct Cells<'a> {
    current_index: usize,
    values: &'a [u8],
}

impl<'a> Iterator for Cells<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let prev_index = self.current_index;
        self.current_index += 1;

        if prev_index < self.values.len() {
            Some(self.values[prev_index])
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Game {
    given_numbers: Numbers,
    current_numbers: Numbers,
    notes: [u16; 81],
}

impl Game {
    pub fn create<N: Into<Numbers>>(given_numbers: N) -> Game {
        let given_numbers = given_numbers.into();
        Game {
            given_numbers,
            current_numbers: given_numbers,
            notes: [0u16; 81],
        }
    }

    pub fn get_notes(&self, row: usize, col: usize) -> u16 {
        self.notes[Self::coords_to_cell_index(row, col)]
    }

    pub fn toggle_note(&mut self, row: usize, col: usize, note: u8) {
        self.notes[Self::coords_to_cell_index(row, col)]
            .bitxor_assign(1u16 << (note - 1).max(0) as i16);
    }

    pub fn cell_index_to_coords(index: usize) -> (usize, usize) {
        assert!(index < 81);
        let col = index % 9;
        let row = (index - col) / 9;
        (row + 1, col + 1)
    }

    pub fn coords_to_cell_index(row: usize, col: usize) -> usize {
        assert!((1..=9).contains(&row));
        assert!((1..=9).contains(&col));
        ((row - 1) * 9) + (col - 1)
    }

    pub fn cells(&self) -> Cells {
        self.current_numbers.cells()
    }

    pub fn index_is_given(&self, index: usize) -> bool {
        self.given_numbers.get_by_offset(index) != 0
    }

    pub fn is_given(&self, row: usize, col: usize) -> bool {
        self.given_numbers.get(row, col) != 0
    }

    pub fn set(&mut self, row: usize, col: usize, value: u8) -> bool {
        if self.is_given(row, col) {
            return false;
        }

        self.current_numbers.set(row, col, value);
        true
    }

    pub fn get(&self, row: usize, col: usize) -> u8 {
        assert!((1..=9).contains(&row));
        assert!((1..=9).contains(&col));

        self.current_numbers.get(row, col)
    }

    pub fn is_valid(&self) -> bool {
        fn nums_uniq(inpt: &[u8]) -> bool {
            let mut seen = [false; 9];
            for &num in inpt {
                if num == 0 {
                    continue;
                }

                let idx = (num - 1) as usize;
                if seen[idx] {
                    return false;
                }

                seen[idx] = true;
            }

            return true;
        }

        // lets first get a naive impl right before trying to be smart
        // check all row
        for group_index in 1..=9 {
            let row = self.current_numbers.get_row(group_index);
            if !nums_uniq(&row) {
                return false;
            }

            let col = self.current_numbers.get_col(group_index);
            if !nums_uniq(&col) {
                return false;
            }

            let sbox = self.current_numbers.get_box(group_index);
            if !nums_uniq(&sbox) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[rustfmt::skip]
    static TEST_FIELD: [u8; 81] = [
        1, 0, 0,  0, 6, 0,  0, 0, 0,
        9, 8, 0,  0, 0, 0,  6, 0, 5,
        0, 0, 0,  0, 0, 5,  0, 0, 1,

        0, 0, 0,  0, 0, 0,  3, 0, 4,
        0, 6, 0,  1, 3, 0,  9, 0, 0,
        0, 4, 0,  7, 2, 0,  0, 0, 0,

        0, 9, 3,  0, 7, 6,  1, 0, 0,
        0, 0, 6,  4, 8, 0,  0, 0, 7,
        5, 0, 0,  9, 0, 2,  4, 6, 0,
    ];

    /// this is a valid field, where we know that inserting another 1 in
    /// any of the free cells will make it immedially invalid, so we can test all 72 cases
    #[rustfmt::skip]
    static VALIDATION_PATTERN: [u8; 81] = [
        1, 0, 0,  0, 0, 0,  0, 0, 0,
        0, 0, 0,  1, 0, 0,  0, 0, 0,
        0, 0, 0,  0, 0, 0,  1, 0, 0,

        0, 1, 0,  0, 0, 0,  0, 0, 0,
        0, 0, 0,  0, 1, 0,  0, 0, 0,
        0, 0, 0,  0, 0, 0,  0, 1, 0,

        0, 0, 1,  0, 0, 0,  0, 0, 0,
        0, 0, 0,  0, 0, 1,  0, 0, 0,
        0, 0, 0,  0, 0, 0,  0, 0, 1,
    ];

    mod numbers {
        use super::*;

        #[test]
        fn get_row_returns_correct_numbers() {
            let n = Numbers::from(TEST_FIELD);

            let row_2 = [9, 8, 0, 0, 0, 0, 6, 0, 5];
            assert_eq!(row_2, n.get_row(2));
        }

        #[test]
        fn get_col_returns_correct_numbers() {
            let n = Numbers::from(TEST_FIELD);

            let col_5 = [6, 0, 0, 0, 3, 2, 7, 8, 0];
            assert_eq!(col_5, n.get_col(5));
        }

        #[test]
        fn get_box_returns_correct_numbers() {
            let n = Numbers::from(TEST_FIELD);
            let box_8 = [0, 7, 6, 4, 8, 0, 9, 0, 2];
            assert_eq!(box_8, n.get_box(8));
        }
    }

    #[test]
    fn test_cell_index_to_coord() {
        assert_eq!((9, 9), Game::cell_index_to_coords(80));
    }

    #[test]
    fn test_puzzle_is_valid_without_inputs() {
        let g = Game::create(TEST_FIELD);
        assert!(g.is_valid());
    }

    #[test]
    fn incorrect_number_in_row_invalid() {
        let mut g = Game::create(TEST_FIELD);
        g.set(1, 4, 1);

        assert_eq!(false, g.is_valid());
    }

    #[test]
    fn incorrect_number_in_col_invalid() {
        let mut g = Game::create(TEST_FIELD);
        g.set(4, 1, 9);

        assert_eq!(false, g.is_valid());
    }

    #[test]
    fn incorrect_number_in_box_invalid() {
        let mut g = Game::create(TEST_FIELD);
        g.set(2, 3, 1);

        assert_eq!(false, g.is_valid());
    }
}
