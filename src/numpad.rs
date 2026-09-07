pub const NUMPAD: [[&str; 3]; 4] = [
    ["1", "2", "3"],
    ["4", "5", "6"],
    ["7", "8", "9"],
    ["DEL", "0", "GO"],
];

pub struct Numpad {
    pub row:   usize,
    pub col:   usize,
    pub entry: String,
}

impl Numpad {
    pub fn new() -> Self {
        Numpad {
            row:   0,
            col:   0,
            entry: String::new(),
        }
    }

    pub fn move_right(&mut self) {
        self.col = (self.col + 1) % 3;
    }

    pub fn move_left(&mut self) {
        self.col = (self.col + 3 - 1) % 3;
    }

    pub fn move_up(&mut self) {
        self.row = (self.row + 4 - 1) % 4;
    }

    pub fn move_down(&mut self) {
        self.row = (self.row + 1) % 4;
    }

    /// Returns target page number if GO was pressed, None otherwise
    pub fn select(&mut self) -> NumpadAction {
        let key = NUMPAD[self.row][self.col];

        match key {
            "GO" => {
                if self.entry.is_empty() {
                    NumpadAction::Cancel
                } else {
                    let page: usize = self.entry.parse().unwrap_or(1);
                    self.entry.clear();
                    NumpadAction::Go(page.saturating_sub(1)) // convert to 0 indexed
                }
            }
            "DEL" => {
                self.entry.pop();
                NumpadAction::Updated
            }
            digit => {
                if self.entry.len() < 6 {
                    self.entry.push_str(digit);
                }
                NumpadAction::Updated
            }
        }
    }

    pub fn cancel(&mut self) {
        self.entry.clear();
        self.row = 0;
        self.col = 0;
    }
}

#[derive(Debug)]
pub enum NumpadAction {
    Go(usize),    // jump to this page
    Updated,      // entry changed, re-render
    Cancel,       // exit numpad
}