use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use crate::battery::Battery;
use crate::buttons::{
    ButtonEvent, ButtonPress, Buttons,
    PIN_BOOK_1, PIN_BOOK_2, PIN_BOOK_3, PIN_BOOK_4,
    PIN_PAGE_BACK, PIN_PAGE_FORWARD,
};
use crate::config::Config;
use crate::display::Display;
use crate::epub::Book;
use crate::library::Library;
use crate::numpad::{Numpad, NumpadAction, NUMPAD};
use crate::settings::{Settings, SettingsAction};

const FONT_PATH: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf";

fn position_file(book_path: &Path) -> PathBuf {
    let mut p        = book_path.to_path_buf();
    let mut filename = p.file_name().unwrap().to_os_string();
    filename.push(".pos");
    p.set_file_name(filename);
    p
}

fn save_position(book_path: &Path, page: usize) {
    let _ = fs::write(position_file(book_path), page.to_string());
}

fn load_position(book_path: &Path) -> usize {
    fs::read_to_string(position_file(book_path))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

enum Mode {
    Reading,
    Library,
    Numpad,
    Settings,
}

pub struct Reader {
    display:      Display,
    buttons:      Buttons,
    battery:      Battery,
    config:       Config,
    book:         Option<Book>,
    book_path:    Option<PathBuf>,
    current_page: usize,
    mode:         Mode,
    numpad:       Numpad,
    library:      Library,
    settings:     Settings,
}

impl Reader {
    pub fn new() -> Result<Self> {
        let font_data = fs::read(FONT_PATH)
            .map_err(|e| anyhow::anyhow!("Failed to load font: {}", e))?;

        let config      = Config::load();
        let mut display = Display::new(font_data)?;
        display.init()?;
        display.clear()?;

        let buttons     = Buttons::new()?;
        let battery     = Battery::new();
        let mut library = Library::new();
        library.scan();

        Ok(Reader {
            display,
            buttons,
            battery,
            config,
            book:         None,
            book_path:    None,
            current_page: 0,
            mode:         Mode::Reading,
            numpad:       Numpad::new(),
            library,
            settings:     Settings::new(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.show_library()?;

        loop {
            if let Some(event) = self.buttons.poll() {
                match self.mode {
                    Mode::Reading  => self.handle_reading(event)?,
                    Mode::Library  => self.handle_library(event)?,
                    Mode::Numpad   => self.handle_numpad(event)?,
                    Mode::Settings => self.handle_settings(event)?,
                }
            }
            sleep(Duration::from_millis(50));
        }
    }

    fn handle_reading(&mut self, event: ButtonEvent) -> Result<()> {
        match event.pin {
            PIN_PAGE_FORWARD => self.page_forward()?,
            PIN_PAGE_BACK    => self.page_back()?,
            PIN_BOOK_1 => match event.press {
                ButtonPress::Short => self.show_library()?,
                ButtonPress::Long  => self.enter_numpad()?,
            },
            PIN_BOOK_2 => match event.press {
                ButtonPress::Short => self.show_library()?,
                ButtonPress::Long  => self.enter_numpad()?,
            },
            PIN_BOOK_3 => match event.press {
                ButtonPress::Short => self.show_library()?,
                ButtonPress::Long  => self.enter_settings()?,
            },
            PIN_BOOK_4 => match event.press {
                ButtonPress::Short => self.show_library()?,
                ButtonPress::Long  => self.shutdown()?,
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_library(&mut self, event: ButtonEvent) -> Result<()> {
        match event.pin {
            PIN_PAGE_FORWARD | PIN_BOOK_2 => {
                self.library.move_down();
                self.render_library()?;
            }
            PIN_PAGE_BACK | PIN_BOOK_1 => {
                self.library.move_up();
                self.render_library()?;
            }
            PIN_BOOK_3 => {
                if let Some(entry) = self.library.selected_book() {
                    let path = entry.path.clone();
                    self.load_book(path)?;
                }
            }
            PIN_BOOK_4 => {
                if self.book.is_some() {
                    self.mode = Mode::Reading;
                    self.render_current_page()?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_numpad(&mut self, event: ButtonEvent) -> Result<()> {
        match event.pin {
            PIN_PAGE_FORWARD => {
                self.numpad.move_right();
                self.render_numpad()?;
            }
            PIN_PAGE_BACK => {
                self.numpad.move_left();
                self.render_numpad()?;
            }
            PIN_BOOK_1 => {
                self.numpad.move_up();
                self.render_numpad()?;
            }
            PIN_BOOK_2 => {
                self.numpad.move_down();
                self.render_numpad()?;
            }
            PIN_BOOK_3 => match self.numpad.select() {
                NumpadAction::Go(page) => {
                    self.exit_numpad()?;
                    self.jump_to_page(page)?;
                }
                NumpadAction::Updated => self.render_numpad()?,
                NumpadAction::Cancel  => self.exit_numpad()?,
            },
            PIN_BOOK_4 => {
                self.numpad.cancel();
                self.exit_numpad()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_settings(&mut self, event: ButtonEvent) -> Result<()> {
        match event.pin {
            PIN_BOOK_1 => {
                self.settings.move_up();
                self.render_settings()?;
            }
            PIN_BOOK_2 => {
                self.settings.move_down();
                self.render_settings()?;
            }
            PIN_PAGE_FORWARD => {
                let action = self.settings.increase(&mut self.config);
                self.handle_settings_action(action)?;
            }
            PIN_PAGE_BACK => {
                let action = self.settings.decrease(&mut self.config);
                self.handle_settings_action(action)?;
            }
            PIN_BOOK_3 => {
                let action = self.settings.select(&mut self.config);
                self.handle_settings_action(action)?;
            }
            PIN_BOOK_4 => {
                let _ = self.config.save();
                self.exit_settings()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_settings_action(&mut self, action: SettingsAction) -> Result<()> {
        match action {
            SettingsAction::Exit => {
                let _ = self.config.save();
                self.exit_settings()?;
            }
            SettingsAction::FontIncreased | SettingsAction::FontDecreased => {
                if let Some(path) = self.book_path.clone() {
                    let saved = self.current_page;
                    self.load_book(path)?;
                    self.current_page = saved;
                }
                self.render_settings()?;
            }
            SettingsAction::None => {
                self.render_settings()?;
            }
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.display.render_message("Shutting down...\nPlease wait.")?;
        sleep(Duration::from_secs(2));
        std::process::Command::new("sudo")
            .args(["shutdown", "-h", "now"])
            .spawn()?;
        Ok(())
    }

    fn page_forward(&mut self) -> Result<()> {
        if let Some(book) = &self.book {
            if self.current_page < book.total_pages() - 1 {
                self.current_page += 1;
                self.save_current_position();
                self.render_current_page()?;
            } else {
                self.display.render_message("End of book")?;
            }
        }
        Ok(())
    }

    fn page_back(&mut self) -> Result<()> {
        if self.book.is_some() {
            if self.current_page > 0 {
                self.current_page -= 1;
                self.save_current_position();
                self.render_current_page()?;
            } else {
                self.display.render_message("Beginning of book")?;
            }
        }
        Ok(())
    }

    fn jump_to_page(&mut self, page: usize) -> Result<()> {
        if let Some(book) = &self.book {
            self.current_page = page.min(book.total_pages() - 1);
            self.save_current_position();
            self.render_current_page()?;
        }
        Ok(())
    }

    fn load_book(&mut self, path: PathBuf) -> Result<()> {
        let msg = format!(
            "Loading:\n{}\nPlease wait...",
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .replace('_', " ")
        );
        self.display.render_message(&msg)?;

        let saved_page = load_position(&path);
        let book       = Book::load(&path, &self.config)?;
        let total      = book.total_pages();

        self.current_page = saved_page.min(total.saturating_sub(1));
        self.book_path    = Some(path);
        self.book         = Some(book);
        self.mode         = Mode::Reading;

        self.render_current_page()
    }

    fn show_library(&mut self) -> Result<()> {
        self.library.scan();
        self.mode = Mode::Library;
        self.render_library()
    }

    fn enter_numpad(&mut self) -> Result<()> {
        self.mode   = Mode::Numpad;
        self.numpad = Numpad::new();
        self.render_numpad()
    }

    fn exit_numpad(&mut self) -> Result<()> {
        self.mode = Mode::Reading;
        if self.book.is_some() {
            self.render_current_page()
        } else {
            self.show_library()
        }
    }

    fn enter_settings(&mut self) -> Result<()> {
        self.mode     = Mode::Settings;
        self.settings = Settings::new();
        self.render_settings()
    }

    fn exit_settings(&mut self) -> Result<()> {
        self.mode = Mode::Reading;
        if self.book.is_some() {
            self.render_current_page()
        } else {
            self.show_library()
        }
    }

    fn render_current_page(&mut self) -> Result<()> {
        if let Some(book) = &self.book {
            if let Some(lines) = book.get_page(self.current_page) {
                let lines       = lines.clone();
                let total_pages = book.total_pages();
                let title       = book.title.clone();
                let battery_str = self.battery.status_string();
                self.display.render_page(
                    &lines,
                    self.current_page,
                    total_pages,
                    &title,
                    &battery_str,
                )?;
            }
        }
        Ok(())
    }

    fn render_library(&mut self) -> Result<()> {
        if self.library.is_empty() {
            return self.display.render_message(
                "No books found\nAdd EPUBs to /home/pi/books/"
            );
        }

        let items: Vec<(String, bool)> = self.library
            .visible_books()
            .iter()
            .enumerate()
            .map(|(i, book)| {
                let selected = self.library.scroll + i == self.library.selected;
                (book.title.clone(), selected)
            })
            .collect();

        let status = format!(
            "Library  {}/{}  BK3=Open BK4=Cancel",
            self.library.selected + 1,
            self.library.count()
        );

        self.display.render_menu("LIBRARY", &items, &status)
    }

    fn render_numpad(&mut self) -> Result<()> {
        self.display.render_numpad(
            &NUMPAD,
            self.numpad.row,
            self.numpad.col,
            &self.numpad.entry.clone(),
        )
    }

    fn render_settings(&mut self) -> Result<()> {
        let items  = self.settings.render_items(&self.config);
        let status = "BK1=Up BK2=Down FWD=+ BACK=- BK3=Select BK4=Save";
        self.display.render_menu("SETTINGS", &items, status)
    }

    fn save_current_position(&self) {
        if let Some(path) = &self.book_path {
            save_position(path, self.current_page);
        }
    }
}