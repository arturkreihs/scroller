use std::io::Write;
use std::sync::RwLock;
use thiserror::Error;
use termion::{
    clear,
    cursor,
    input::TermRead,
    event::Key,
    raw::IntoRawMode,
};

#[derive(Error, Debug)]
pub enum ScrollerError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("RwLock was poisoned")]
    RwLockPoisoned,
}

pub struct Scroller {
    screen: termion::raw::RawTerminal<std::io::Stdout>,
    rows: u16,
    buffer: RwLock<Vec<char>>,
}

impl Default for Scroller {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Scroller {
    pub fn new() -> Result<Self, ScrollerError> {
        let screen = std::io::stdout().into_raw_mode()?;
        let (_, rows) = termion::terminal_size()?;
        
        // prepare screen
        {
            let mut screen = screen.lock();
            write!(screen, "{}", clear::All)?;
            write!(screen, "{}", cursor::Goto(1, rows))?;
            screen.flush()?;
        }

        Ok(Scroller {
            screen,
            rows,
            buffer: RwLock::new(Vec::<char>::new()),
        })
    }

    pub fn write(&self, line: &str) -> Result<(), ScrollerError> {
        // take screen here
        let mut screen = self.screen.lock();

        // save current position
        write!(screen, "{}", termion::cursor::Save)?;

        // scroll up
        write!(screen, "{}", termion::scroll::Up(1))?;

        // go to bottom line - 1
        write!(screen, "{}", cursor::Goto(1, self.rows - 1))?;

        // write text to that line
        write!(screen, "{}", line)?;
        
        // clear the rest (rubbish after scrool::Up)
        write!(screen, "{}", clear::AfterCursor)?;

        // set cursor to the bottom line
        write!(screen, "{}", cursor::Goto(1, self.rows))?;

        // write unprocessed user input
        let buffer = self.buffer.read().map_err(|_| ScrollerError::RwLockPoisoned)?;
        write!(screen, "{}", buffer.iter().collect::<String>())?;

        screen.flush()?;
        Ok(())
    }

    pub fn read(&self) -> Result<Option<String>, ScrollerError> {
        for key in std::io::stdin().keys() {
            // take screen here
            let mut screen = self.screen.lock();

            // take input buf here
            let mut buffer = self.buffer.write().map_err(|_| ScrollerError::RwLockPoisoned)?;

            match key? {
                // clear line and do action on enter key
                Key::Char('\n') => {
                    write!(screen, "{}", cursor::Goto(1, self.rows))?;
                    write!(screen, "{}", clear::CurrentLine)?;
                    let line = buffer.iter().collect();
                    buffer.clear();
                    screen.flush()?;
                    return Ok(Some(line));
                },

                // add char to buffer and print it
                Key::Char(c) => {
                    write!(screen, "{}", c)?;
                    buffer.push(c);
                },

                // go one char back and clear it
                Key::Backspace => {
                    write!(screen, "{}", cursor::Left(1))?;
                    write!(screen, "{}", clear::AfterCursor)?;
                    buffer.pop();
                },

                // exit loop
                Key::Ctrl('c') => {
                    break;
                }

                // anything else will continue
                _ => continue,
            }
            screen.flush()?;
        }
        Ok(None)
    }
}

impl Drop for Scroller {
    fn drop(&mut self) {
        self.screen.suspend_raw_mode().unwrap();
    }
}
