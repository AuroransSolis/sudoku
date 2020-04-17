use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CellValue {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
}

impl CellValue {
    pub fn new(n: u8) -> Option<Self> {
        if n == 0 || n > 9 {
            None
        } else {
            Some(match n {
                1 => CellValue::One,
                2 => CellValue::Two,
                3 => CellValue::Three,
                4 => CellValue::Four,
                5 => CellValue::Five,
                6 => CellValue::Six,
                7 => CellValue::Seven,
                8 => CellValue::Eight,
                9 => CellValue::Nine,
                _ => unreachable!(),
            })
        }
    }
}

impl From<CellValue> for usize {
    fn from(other: CellValue) -> Self {
        (other as u8 - 1) as usize
    }
}

#[derive(Copy, Clone)]
// Each board is an array of rows (reverse coordinates, (y, x))
pub struct Game {
    board: [[Option<CellValue>; 9]; 9],
    cell_poss: [[[bool; 9]; 9]; 9],
    pub cols_flags: [[bool; 9]; 9],
    pub rows_flags: [[bool; 9]; 9],
    pub sqrs_flags: [[bool; 9]; 9],
}

impl Game {
    pub fn new(numbers: [[u8; 9]; 9]) -> Self {
        let mut board = [[None; 9]; 9];
        let mut cell_poss = [[[true; 9]; 9]; 9];
        // Arrays of markers for whether each group has a cell value yet
        let mut rows_flags = [[false; 9]; 9];
        let mut cols_flags = [[false; 9]; 9];
        let mut sqrs_flags = [[false; 9]; 9];
        for (y, row) in rows_flags.iter_mut().enumerate() {
            for (x, col) in cols_flags.iter_mut().enumerate() {
                let n = numbers[y][x];
                assert!(n < 10);
                if let Some(cv) = CellValue::new(n) {
                    // Mark everything but the stored value impossible
                    for i in (0..9).filter(|&i| i != n as usize - 1) {
                        cell_poss[y][x][i] = false;
                    }
                    board[y][x] = Some(cv);
                    let s = 3 * (y / 3) + x / 3;
                    row[n as usize - 1] = true;
                    col[n as usize - 1] = true;
                    sqrs_flags[s][n as usize - 1] = true;
                }
            }
        }
        // Update possibility arrays for unset cells, which is equivalent to updating possibility
        // arrays that have everything marked as possible.
        for (y, row) in rows_flags.iter().enumerate() {
            for (x, col) in cols_flags.iter().enumerate() {
                if board[y][x].is_none() {
                    for i in 0..9 {
                        let s = 3 * (y / 3) + x / 3;
                        cell_poss[y][x][i] = !(row[i] || col[i] || sqrs_flags[s][i]);
                    }
                }
            }
        }
        let new = Game {
            board,
            cell_poss,
            cols_flags,
            rows_flags,
            sqrs_flags,
        };
        assert!(new.is_valid(true));
        new
    }

    fn iter(&self) -> impl Iterator<Item = (usize, usize, &Option<CellValue>, &[bool; 9])> + '_ {
        (0..9)
            .flat_map(|y| (0..9).map(move |x| (y, x)))
            .map(move |(y, x)| (y, x, &self.board[y][x], &self.cell_poss[y][x]))
    }

    fn iter_cells(&self) -> impl Iterator<Item = (usize, usize, &Option<CellValue>)> + '_ {
        self.board
            .iter()
            .enumerate()
            .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, n)| (y, x, n)))
    }

    fn iter_row_poss(&self, row: usize) -> impl Iterator<Item = (usize, &[bool; 9])> + '_ {
        self.cell_poss[row].iter().enumerate()
    }

    fn iter_col_poss(&self, col: usize) -> impl Iterator<Item = (usize, &[bool; 9])> + '_ {
        self.cell_poss.iter().map(move |row| &row[col]).enumerate()
    }

    fn iter_3x3_poss(
        &self,
        row: usize,
        col: usize,
    ) -> impl Iterator<Item = (usize, usize, &[bool; 9])> + '_ {
        let r_start = 3 * (row / 3);
        let c_start = 3 * (col / 3);
        self.cell_poss[r_start..r_start + 3]
            .iter()
            .enumerate()
            .map(move |(y, r)| (y + r_start, r))
            .flat_map(move |(y, r)| {
                r[c_start..c_start + 3]
                    .iter()
                    .enumerate()
                    .map(move |(x, n)| (y, x + c_start, n))
            })
    }

    fn set_cell(&mut self, row: usize, col: usize, cv: CellValue) {
        if self.board[row][col] == Some(cv) {
            return;
        }
        self.board[row][col] = Some(cv);
        let mut set = [false; 9];
        set[usize::from(cv)] = true;
        self.cell_poss[row][col] = set;
        self.cols_flags[col][usize::from(cv)] = true;
        self.rows_flags[row][usize::from(cv)] = true;
        let s = self.sqrs_ind(row, col);
        self.sqrs_flags[s][usize::from(cv)] = true;
        self.update_poss_from_flags(row, col);
    }

    fn unset_cell(&mut self, row: usize, col: usize) {
        let i = match self.board[row][col] {
            Some(cv) => usize::from(cv),
            None => return,
        };
        self.board[row][col] = None;
        self.cols_flags[col][i] = false;
        self.rows_flags[row][i] = false;
        let s = self.sqrs_ind(row, col);
        self.sqrs_flags[s][i] = false;
        self.update_poss_from_flags(row, col);
    }

    fn update_poss_from_flags(&mut self, row: usize, col: usize) {
        // Set the new possibilities for the affected row
        for (x, c) in self.cols_flags.iter().enumerate() {
            if self.board[row][x].is_none() {
                let s = 3 * (row / 3) + x / 3;
                for (i, &c) in c.iter().enumerate() {
                    let new = !(self.rows_flags[row][i] || c || self.sqrs_flags[s][i]);
                    self.cell_poss[row][x][i] = new;
                }
            }
        }
        // Set the new possibilities for the affected column (less the removed cell, which was fixed
        // by the previous loop)
        for (y, r) in self
            .rows_flags
            .iter()
            .enumerate()
            .filter(|&(y, _)| y != row)
        {
            if self.board[y][col].is_none() {
                let s = 3 * (y / 3) + col / 3;
                for (i, &r) in r.iter().enumerate() {
                    let new = !(r || self.cols_flags[col][i] || self.sqrs_flags[s][i]);
                    self.cell_poss[y][col][i] = new;
                }
            }
        }
        // There should be four more cells in the 3x3 group not fixed by the previous two loops.
        // Iterate over them and fix their possibilities.
        let rs = 3 * (row / 3);
        let cs = 3 * (col / 3);
        for y in (rs..rs + 3).filter(|&y| y != row) {
            for x in (cs..cs + 3).filter(|&x| x != col) {
                if self.board[y][x].is_none() {
                    let s = 3 * (y / 3) + x / 3;
                    for i in 0..9 {
                        let new = !(self.rows_flags[y][i]
                            || self.cols_flags[x][i]
                            || self.sqrs_flags[s][i]);
                        self.cell_poss[y][x][i] = new;
                    }
                }
            }
        }
    }

    pub fn propagate_poss_to_board(&mut self) -> bool {
        // Only try to make changes if the game isn't already solved
        if !self.solved() {
            let mut made_change = false;
            for cv in 0..9 {
                if self.rows_flags.iter().filter(|b| b[cv]).count() == 8 {
                    let r = self.rows_flags.iter().position(|b| !b[cv]).expect("rfr");
                    let c = self.iter_row_poss(r).position(|(_, cell)| cell[cv]);
                    if let Some(c) = c {
                        self.set_cell(r, c, CellValue::new(cv as u8 + 1).expect("rfcv"));
                        made_change = true;
                    }
                }
                if self.cols_flags.iter().filter(|b| b[cv]).count() == 8 {
                    let c = self.cols_flags.iter().position(|b| !b[cv]).expect("cfc");
                    let r = self.iter_col_poss(c).position(|(_, cell)| cell[cv]);
                    if let Some(r) = r {
                        self.set_cell(r, c, CellValue::new(cv as u8 + 1).expect("cfcv"));
                        made_change = true;
                    }
                }
                if self.sqrs_flags.iter().filter(|b| b[cv]).count() == 8 {
                    let s = self.sqrs_flags.iter().position(|b| !b[cv]).expect("sfs");
                    let rs = 3 * (s / 3);
                    let cs = 3 * (s % 3);
                    let p = self
                        .iter_3x3_poss(rs, cs)
                        .position(|(_, _, cell)| cell[cv]);
                    if let Some(p) = p {
                        let ro = p / 3;
                        let co = p % 3;
                        let r = rs + ro;
                        let c = cs + co;
                        self.set_cell(r, c, CellValue::new(cv as u8 + 1).expect("sfcv"));
                        made_change = true;
                    }
                }
            }
            for y in 0..9 {
                for x in 0..9 {
                    // Only check possibilities if the board has no value in a cell
                    if self.board[y][x].is_none() {
                        if self.cell_poss[y][x].iter().copied().filter(|&b| b).count() == 1 {
                            let cv = self.cell_poss[y][x].iter().position(|&b| b).unwrap();
                            self.set_cell(y, x, CellValue::new(cv as u8 + 1).expect("xcv"));
                            made_change = true;
                        }
                    }
                }
            }
            made_change
        } else {
            false
        }
    }

    pub fn solve(&mut self) {
        if self.solved() {
            return;
        }
        // Solve as much of the puzzle as is possible without any sort of foresight - just cancel
        // out possible values and put in values for cells with only one possible value for as long
        // as possible.
        loop {
            if !self.propagate_poss_to_board() {
                break;
            }
        }
        // If this solves the puzzle, hooray! Easy win, just return.
        if self.solved() {
            return;
        }
        // Each level of recursion represents a single move. So the maximum level of recursion is
        // the number of moves left to make. It shouldn't be possible to go over this cap, but this
        // is here as a precaution to keep the program from overrunning the stack.
        let depth_cap = 81
            - self
                .board
                .iter()
                .map(|row| row.iter().filter(|cv| cv.is_some()).count())
                .sum::<usize>();
        // Get the coordinates and possibilities for the first cell with more than one possible
        // value.
        let (y, x, poss) = self
            .iter()
            .find(|&(_, _, cell, _)| cell.is_none())
            .map(|(y, x, _, &poss)| (y, x, poss))
            .unwrap();
        // Iterate over the possible values the cell can be.
        for cv in poss
            .iter()
            .enumerate()
            .filter(|&(_, &p)| p)
            .map(|(i, _)| CellValue::new(i as u8 + 1).unwrap())
        {
            let mut new = *self;
            // Set the cell to the possible value
            new.set_cell(y, x, cv);
            // Make sure that this change is valid (especially that it leaves possibilities).
            if !new.is_valid(false) {
                new.unset_cell(y, x);
                continue;
            }
            // If that move solved the game, return.
            if new.solved() {
                *self = new;
                return;
            }
            // If it didn't, this becomes the base of a recursive walk over the possible moves for
            // the game with that as the starting point. If this tree produces a solved game (the
            // recursive call returns `true`), then return. Otherwise, undo the move and try the
            // next one.
            if new.solve_recursive(1, depth_cap) {
                *self = new;
                return;
            }
        }
        panic!("Found no unique solution to game.");
    }

    fn solve_recursive(&mut self, depth: usize, max_depth: usize) -> bool {
        if depth > max_depth {
            return false;
        }
        // Solve as much of the puzzle as is possible without any sort of foresight - just cancel
        // out possible values and put in values for cells with only one possible value for as long
        // as possible.
        loop {
            if !self.propagate_poss_to_board() {
                break;
            }
        }
        // If this solves the puzzle, hooray! Easy win, just return.
        if self.solved() {
            return true;
        }
        // Get the coordinates and possibilities for the first cell with more than one possible
        // value. The `unwrap` is safe here since this method never gets called if there are no
        // empty cells left.
        let (y, x, poss) = self
            .iter()
            .find(|&(_, _, cell, _)| cell.is_none())
            .map(|(y, x, _, &poss)| (y, x, poss))
            .unwrap();
        // Iterate over the possible values the cell can be and branch to all the possible moves
        // after this one. If a move solves the game or if a branch returns true, return `true`
        // immediately to walk back up the stack to the base of the tree and return. If a branch
        // returns false, try the next one. If all branches are exhausted and no solution has been
        // found, then this is a bad branch so return `false`.
        for cv in poss
            .iter()
            .enumerate()
            .filter(|&(_, &p)| p)
            .map(|(i, _)| CellValue::new(i as u8 + 1).unwrap())
        {
            let mut new = *self;
            new.set_cell(y, x, cv);
            if !new.is_valid(false) {
                new.unset_cell(y, x);
                continue;
            }
            if new.solved() {
                return true;
            }
            if new.solve_recursive(depth + 1, max_depth) {
                *self = new;
                return true;
            } else {
            }
        }
        false
    }

    fn solved(&self) -> bool {
        // Keep flags for whether each row, column, or 3x3 has a certain cell value.
        let mut rows = [[false; 9]; 9];
        let mut cols = [[false; 9]; 9];
        let mut sqrs = [[false; 9]; 9];
        for (y, x, &cell) in self.iter_cells() {
            match cell {
                Some(cv) => {
                    let i = usize::from(cv);
                    let sqrs_x = x / 3;
                    let sqrs_y = 3 * (y / 3);
                    if rows[y][i] || cols[x][i] || sqrs[sqrs_x + sqrs_y][i] {
                        return false;
                    } else {
                        rows[y][i] = true;
                        cols[x][i] = true;
                        sqrs[sqrs_x + sqrs_y][i] = true;
                    }
                }
                None => return false,
            }
        }
        true
    }

    fn is_valid(&self, verbose: bool) -> bool {
        // Make sure there aren't any unset cells with no possible values
        if let Some((y, x, _, _)) = self
            .iter()
            .find(|&(_, _, cell, poss)| cell.is_none() && !poss.contains(&true))
        {
            if verbose {
                println!("Cell ({}, {}) has no possible values", x, y);
            }
            false
        } else {
            // Otherwise, check to make sure there are no conflicts in rows, columns, or 3x3s.
            let mut rows = [[false; 9]; 9];
            let mut cols = [[false; 9]; 9];
            let mut sqrs = [[false; 9]; 9];
            for (y, x, &cell) in self.iter_cells() {
                if let Some(cv) = cell {
                    let i = usize::from(cv);
                    let s = 3 * (y / 3) + x / 3;
                    if rows[y][i] || cols[x][i] || sqrs[s][i] {
                        if verbose {
                            if rows[y][i] {
                                println!("Conflict: row {} has multiple {}s", y, i + 1);
                            }
                            if cols[x][i] {
                                println!("Conflict: col {} has multiple {}s", x, i + 1);
                            }
                            if sqrs[s][i] {
                                println!("Conflict: sqr {} has multiple {}s", s, i + 1);
                            }
                        }
                        return false;
                    } else {
                        rows[y][i] = true;
                        cols[x][i] = true;
                        sqrs[s][i] = true;
                    }
                }
            }
            true
        }
    }

    fn sqrs_ind(&self, row: usize, col: usize) -> usize {
        3 * (row / 3) + col / 3
    }

    fn cell_char(&self, row: usize, col: usize) -> char {
        match self.board[row][col].map(|n| n as u8) {
            Some(1) => '1',
            Some(2) => '2',
            Some(3) => '3',
            Some(4) => '4',
            Some(5) => '5',
            Some(6) => '6',
            Some(7) => '7',
            Some(8) => '8',
            Some(9) => '9',
            None => ' ',
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "┌───┬───┬───╥───┬───┬───╥───┬───┬───┐",)?;
        writeln!(
            f,
            "│ {} │ {} │ {} ║ {} │ {} │ {} ║ {} │ {} │ {} │",
            self.cell_char(0, 0),
            self.cell_char(0, 1),
            self.cell_char(0, 2),
            self.cell_char(0, 3),
            self.cell_char(0, 4),
            self.cell_char(0, 5),
            self.cell_char(0, 6),
            self.cell_char(0, 7),
            self.cell_char(0, 8)
        )?;
        writeln!(f, "├───┼───┼───╫───┼───┼───╫───┼───┼───┤",)?;
        writeln!(
            f,
            "│ {} │ {} │ {} ║ {} │ {} │ {} ║ {} │ {} │ {} │",
            self.cell_char(1, 0),
            self.cell_char(1, 1),
            self.cell_char(1, 2),
            self.cell_char(1, 3),
            self.cell_char(1, 4),
            self.cell_char(1, 5),
            self.cell_char(1, 6),
            self.cell_char(1, 7),
            self.cell_char(1, 8)
        )?;
        writeln!(f, "├───┼───┼───╫───┼───┼───╫───┼───┼───┤",)?;
        writeln!(
            f,
            "│ {} │ {} │ {} ║ {} │ {} │ {} ║ {} │ {} │ {} │",
            self.cell_char(2, 0),
            self.cell_char(2, 1),
            self.cell_char(2, 2),
            self.cell_char(2, 3),
            self.cell_char(2, 4),
            self.cell_char(2, 5),
            self.cell_char(2, 6),
            self.cell_char(2, 7),
            self.cell_char(2, 8)
        )?;
        writeln!(f, "╞═══╪═══╪═══╬═══╪═══╪═══╬═══╪═══╪═══╡",)?;
        writeln!(
            f,
            "│ {} │ {} │ {} ║ {} │ {} │ {} ║ {} │ {} │ {} │",
            self.cell_char(3, 0),
            self.cell_char(3, 1),
            self.cell_char(3, 2),
            self.cell_char(3, 3),
            self.cell_char(3, 4),
            self.cell_char(3, 5),
            self.cell_char(3, 6),
            self.cell_char(3, 7),
            self.cell_char(3, 8)
        )?;
        writeln!(f, "├───┼───┼───╫───┼───┼───╫───┼───┼───┤",)?;
        writeln!(
            f,
            "│ {} │ {} │ {} ║ {} │ {} │ {} ║ {} │ {} │ {} │",
            self.cell_char(4, 0),
            self.cell_char(4, 1),
            self.cell_char(4, 2),
            self.cell_char(4, 3),
            self.cell_char(4, 4),
            self.cell_char(4, 5),
            self.cell_char(4, 6),
            self.cell_char(4, 7),
            self.cell_char(4, 8)
        )?;
        writeln!(f, "├───┼───┼───╫───┼───┼───╫───┼───┼───┤",)?;
        writeln!(
            f,
            "│ {} │ {} │ {} ║ {} │ {} │ {} ║ {} │ {} │ {} │",
            self.cell_char(5, 0),
            self.cell_char(5, 1),
            self.cell_char(5, 2),
            self.cell_char(5, 3),
            self.cell_char(5, 4),
            self.cell_char(5, 5),
            self.cell_char(5, 6),
            self.cell_char(5, 7),
            self.cell_char(5, 8)
        )?;
        writeln!(f, "╞═══╪═══╪═══╬═══╪═══╪═══╬═══╪═══╪═══╡",)?;
        writeln!(
            f,
            "│ {} │ {} │ {} ║ {} │ {} │ {} ║ {} │ {} │ {} │",
            self.cell_char(6, 0),
            self.cell_char(6, 1),
            self.cell_char(6, 2),
            self.cell_char(6, 3),
            self.cell_char(6, 4),
            self.cell_char(6, 5),
            self.cell_char(6, 6),
            self.cell_char(6, 7),
            self.cell_char(6, 8)
        )?;
        writeln!(f, "├───┼───┼───╫───┼───┼───╫───┼───┼───┤",)?;
        writeln!(
            f,
            "│ {} │ {} │ {} ║ {} │ {} │ {} ║ {} │ {} │ {} │",
            self.cell_char(7, 0),
            self.cell_char(7, 1),
            self.cell_char(7, 2),
            self.cell_char(7, 3),
            self.cell_char(7, 4),
            self.cell_char(7, 5),
            self.cell_char(7, 6),
            self.cell_char(7, 7),
            self.cell_char(7, 8)
        )?;
        writeln!(f, "├───┼───┼───╫───┼───┼───╫───┼───┼───┤",)?;
        writeln!(
            f,
            "│ {} │ {} │ {} ║ {} │ {} │ {} ║ {} │ {} │ {} │",
            self.cell_char(8, 0),
            self.cell_char(8, 1),
            self.cell_char(8, 2),
            self.cell_char(8, 3),
            self.cell_char(8, 4),
            self.cell_char(8, 5),
            self.cell_char(8, 6),
            self.cell_char(8, 7),
            self.cell_char(8, 8)
        )?;
        writeln!(f, "└───┴───┴───╨───┴───┴───╨───┴───┴───┘",)?;
        Ok(())
    }
}
