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
    possibilities: [[[bool; 9]; 9]; 9],
}

impl Game {
    pub fn new(numbers: [[u8; 9]; 9]) -> Self {
        let mut board = [[None; 9]; 9];
        let mut possibilities = [[[true; 9]; 9]; 9];
        let mut rows = [[false; 9]; 9];
        let mut cols = [[false; 9]; 9];
        let mut sqrs = [[false; 9]; 9];
        for (y, row) in rows.iter_mut().enumerate() {
            for (x, col) in cols.iter_mut().enumerate() {
                let n = numbers[y][x];
                assert!(n < 10);
                if let Some(cv) = CellValue::new(n) {
                    // Mark everything but the stored value impossible
                    for i in (0..9).filter(|&i| i != n as usize - 1) {
                        possibilities[y][x][i] = false;
                    }
                    board[y][x] = Some(cv);
                    let s = 3 * (y / 3) + x / 3;
                    row[n as usize - 1] = true;
                    col[n as usize - 1] = true;
                    sqrs[s][n as usize - 1] = true;
                }
            }
        }
        // Update possibility arrays for unset cells, which is equivalent to updating possibility
        // arrays that have everything marked as possible.
        for y in 0..9 {
            for x in 0..9 {
                if board[y][x].is_none() {
                    for i in 0..9 {
                        let s = 3 * (y / 3) + x / 3;
                        possibilities[y][x][i] = !(rows[y][i] || cols[x][i] || sqrs[s][i]);
                    }
                }
            }
        }
        let new = Game {
            board,
            possibilities,
        };
        assert!(new.is_valid(true));
        new
    }

    fn iter(&self) -> impl Iterator<Item = (usize, usize, &Option<CellValue>, &[bool; 9])> + '_ {
        (0..9)
            .flat_map(|y| (0..9).map(move |x| (y, x)))
            .map(move |(y, x)| (y, x, &self.board[y][x], &self.possibilities[y][x]))
    }

    fn iter_cells(&self) -> impl Iterator<Item = (usize, usize, &Option<CellValue>)> + '_ {
        self.board
            .iter()
            .enumerate()
            .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, n)| (y, x, n)))
    }

    fn iter_row_poss_mut(
        &mut self,
        row: usize,
    ) -> impl Iterator<Item = (usize, &mut [bool; 9])> + '_ {
        self.possibilities[row].iter_mut().enumerate()
    }

    fn iter_col_poss_mut(
        &mut self,
        col: usize,
    ) -> impl Iterator<Item = (usize, &mut [bool; 9])> + '_ {
        self.possibilities
            .iter_mut()
            .map(move |row| &mut row[col])
            .enumerate()
    }

    fn iter_3x3_poss_mut(
        &mut self,
        row: usize,
        col: usize,
    ) -> impl Iterator<Item = (usize, usize, &mut [bool; 9])> + '_ {
        let r_start = 3 * (row / 3);
        let c_start = 3 * (col / 3);
        self.possibilities[r_start..r_start + 3]
            .iter_mut()
            .enumerate()
            .map(move |(y, r)| (y + r_start, r))
            .flat_map(move |(y, r)| {
                r[c_start..c_start + 3]
                    .iter_mut()
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
        self.possibilities[row][col] = set;
        for (_, poss) in self.iter_row_poss_mut(row).filter(|&(c, _)| c != col) {
            poss[usize::from(cv)] = false;
        }
        for (_, poss) in self.iter_col_poss_mut(col).filter(|&(r, _)| r != row) {
            poss[usize::from(cv)] = false;
        }
        for (_, _, poss) in self
            .iter_3x3_poss_mut(row, col)
            .filter(|&(r, c, _)| !(r == row && c == col))
        {
            poss[usize::from(cv)] = false;
        }
    }

    fn unset_cell(&mut self, row: usize, col: usize) {
        self.board[row][col] = None;
        let mut rows = [[false; 9]; 9];
        let mut cols = [[false; 9]; 9];
        let mut sqrs = [[false; 9]; 9];
        for (y, row) in rows.iter_mut().enumerate() {
            for (x, col) in cols.iter_mut().enumerate() {
                if let Some(cv) = self.board[y][x] {
                    let i = usize::from(cv);
                    let s = 3 * (y / 3) + x / 3;
                    row[i] = true;
                    col[i] = true;
                    sqrs[s][i] = true;
                }
            }
        }
        for x in 0..9 {
            if self.board[row][x].is_none() {
                let s = 3 * (row / 3) + x / 3;
                for i in 0..9 {
                    self.possibilities[row][x][i] = !(rows[row][i] || cols[x][i] || sqrs[s][i]);
                }
            }
        }
        for y in (0..9).filter(|&y| y != row) {
            if self.board[y][col].is_none() {
                let s = 3 * (y / 3) + col / 3;
                for i in 0..9 {
                    self.possibilities[y][col][i] = !(rows[y][i] || cols[col][i] || sqrs[s][i]);
                }
            }
        }
        let rs = 3 * (row / 3);
        let cs = 3 * (col / 3);
        for y in (rs..rs + 3).filter(|&y| y != row) {
            for x in (cs..cs + 3).filter(|&x| x != col) {
                if self.board[y][x].is_none() {
                    let s = 3 * (y / 3) + x / 3;
                    for i in 0..9 {
                        self.possibilities[y][x][i] = !(rows[y][i] || cols[x][i] || sqrs[s][i]);
                    }
                }
            }
        }
    }

    fn propagate_poss_to_board(&mut self) -> bool {
        if !self.solved() {
            let mut made_change = false;
            for y in 0..9 {
                'x: for x in 0..9 {
                    if self.board[y][x].is_some() {
                        continue;
                    }
                    let mut n = 0;
                    for i in 0..9 {
                        if self.possibilities[y][x][i] {
                            if n > 0 {
                                continue 'x;
                            } else {
                                n = i + 1;
                            }
                        }
                    }
                    if n > 0 {
                        self.set_cell(y, x, CellValue::new(n as u8).unwrap());
                        made_change = true;
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
        let mut propagated = self.propagate_poss_to_board();
        while propagated {
            propagated = self.propagate_poss_to_board();
        }
        if self.solved() {
            return;
        }
        let depth_cap = 81
            - self
                .board
                .iter()
                .map(|row| row.iter().filter(|cv| cv.is_some()).count())
                .sum::<usize>();
        let (y, x, poss) = self
            .iter()
            .find(|&(_, _, cell, _)| cell.is_none())
            .map(|(y, x, _, &poss)| (y, x, poss))
            .unwrap();
        for cv in poss
            .iter()
            .enumerate()
            .filter(|&(_, &p)| p)
            .map(|(i, _)| CellValue::new(i as u8 + 1).unwrap())
        {
            self.set_cell(y, x, cv);
            let mut propagated = self.propagate_poss_to_board();
            while propagated {
                propagated = self.propagate_poss_to_board();
            }
            if !self.is_valid(false) {
                self.unset_cell(y, x);
                continue;
            }
            if self.solved() {
                return;
            }
            if self.solve_recursive(1, depth_cap) {
                return;
            } else {
                self.unset_cell(y, x);
            }
        }
        panic!("Found no unique solution to game.");
    }

    fn solve_recursive(&mut self, depth: usize, max_depth: usize) -> bool {
        if depth > max_depth {
            println!("hit depth cap, climbing back up.");
            return false;
        }
        let find = self
            .iter()
            .find(|&(_, _, cell, _)| cell.is_none())
            .map(|(y, x, _, &poss)| (y, x, poss));
        let (y, x, poss) = match find {
            Some(details) => details,
            None => return false,
        };
        for cv in poss
            .iter()
            .enumerate()
            .filter(|&(_, &p)| p)
            .map(|(i, _)| CellValue::new(i as u8 + 1).unwrap())
        {
            self.set_cell(y, x, cv);
            if !self.is_valid(false) {
                self.unset_cell(y, x);
                continue;
            }
            if self.solved() {
                return true;
            }
            if self.solve_recursive(depth + 1, max_depth) {
                return true;
            } else {
                self.unset_cell(y, x);
            }
        }
        false
    }

    fn solved(&self) -> bool {
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
        if let Some((y, x, _, _)) = self
            .iter()
            .find(|&(_, _, cell, poss)| cell.is_none() && !poss.contains(&true))
        {
            if verbose {
                println!("cell ({}, {}) has no possible values", x, y);
            }
            false
        } else {
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
                                println!("conflict: row {} has multiple {}s", y, i + 1);
                            }
                            if cols[x][i] {
                                println!("conflict: col {} has multiple {}s", x, i + 1);
                            }
                            if sqrs[sqrs_x + sqrs_y][i] {
                                println!(
                                    "conflict: sqr {} has multiple {}s",
                                    sqrs_x + sqrs_y,
                                    i + 1
                                );
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

    pub fn cell_char(&self, row: usize, col: usize) -> char {
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
