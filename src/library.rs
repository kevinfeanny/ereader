use std::fs;
use std::path::{Path, PathBuf};

const BOOKS_PATH: &str = "/home/pi/books/";

#[derive(Debug, Clone)]
pub struct BookEntry {
    pub title:    String,
    pub path:     PathBuf,
    pub filename: String,
}

pub struct Library {
    pub books:    Vec<BookEntry>,
    pub selected: usize,
    pub scroll:   usize,      // first visible item
    pub visible:  usize,      // how many items fit on screen
}

impl Library {
    pub fn new() -> Self {
        Library {
            books:    Vec::new(),
            selected: 0,
            scroll:   0,
            visible:  8,  // adjust based on font size later
        }
    }

    /// Scan books folder and load all EPUBs
    pub fn scan(&mut self) {
        self.books.clear();

        let path = Path::new(BOOKS_PATH);
        if !path.exists() {
            return;
        }

        let mut entries: Vec<PathBuf> = fs::read_dir(path)
            .unwrap_or_else(|_| panic!("Cannot read books directory"))
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .map(|ext| ext == "epub")
                    .unwrap_or(false)
            })
            .collect();

        // Sort alphabetically
        entries.sort();

        for path in entries {
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Clean up title from filename
            let title = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .replace('_', " ")
                .to_string();

            self.books.push(BookEntry {
                title,
                path,
                filename,
            });
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            // Scroll up if needed
            if self.selected < self.scroll {
                self.scroll = self.selected;
            }
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected + 1 < self.books.len() {
            self.selected += 1;
            // Scroll down if needed
            if self.selected >= self.scroll + self.visible {
                self.scroll = self.selected - self.visible + 1;
            }
        }
    }

    /// Get currently selected book
    pub fn selected_book(&self) -> Option<&BookEntry> {
        self.books.get(self.selected)
    }

    /// Get visible books for rendering
    pub fn visible_books(&self) -> &[BookEntry] {
        let end = (self.scroll + self.visible).min(self.books.len());
        &self.books[self.scroll..end]
    }

    /// Is the library empty?
    pub fn is_empty(&self) -> bool {
        self.books.is_empty()
    }

    /// Total book count
    pub fn count(&self) -> usize {
        self.books.len()
    }
}