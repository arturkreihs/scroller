use std::io::Write;
use std::io::stdin;
use termion::{
    clear,
    cursor,
    input::TermRead,
    event::Key,
    raw::IntoRawMode,
};
use thiserror::Error;
use std::sync::Mutex;
use std::cell::RefCell;

#[derive(Error, Debug)]
pub enum ScrollerError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

pub struct Scroller {
    screen: termion::raw::RawTerminal<std::io::Stdout>,
    rows: u16,
    input: Mutex<RefCell<Vec<char>>>,
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
            input: Mutex::new(RefCell::new(Vec::<char>::new())),
        })
    }

    pub async fn write(&self, line: &str) -> Result<(), ScrollerError> {
        let mut screen = self.screen.lock();
        // save current position
        write!(screen, "{}", termion::cursor::Save).unwrap();

        // scroll up
        write!(screen, "{}", termion::scroll::Up(1)).unwrap();

        // go to bottom line - 1
        write!(screen, "{}", cursor::Goto(1, self.rows - 1)).unwrap();

        // write text to that line
        write!(screen, "{}", line).unwrap();
        
        // clear the rest
        write!(screen, "{}", clear::AfterCursor).unwrap();

        write!(screen, "{}", cursor::Goto(1, self.rows)).unwrap();

        let input = self.input.lock().unwrap();
        let line = input.borrow();
        write!(screen, "{}", line.iter().collect::<String>()).unwrap();

        // restore position
        write!(screen, "{}", termion::cursor::Restore).unwrap();

        screen.flush().unwrap();
        Ok(())
    }

    pub async fn read(&self) -> Result<Option<String>, ScrollerError> {
        for key in stdin().keys() {
            // take stdout here
            let mut screen = self.screen.lock();

            // take input buf here
            let input = self.input.lock().unwrap();
            let mut line = input.borrow_mut();

            match key? {
                // clear line and do action on enter
                Key::Char('\n') => {
                    write!(screen, "{}", cursor::Goto(1, self.rows))?;
                    write!(screen, "{}", clear::CurrentLine)?;
                    let val = line.iter().collect();
                    line.clear();
                    return Ok(Some(val));
                },

                // add char to buffer and print
                Key::Char(c) => {
                    write!(screen, "{}", c)?;
                    line.push(c);
                },

                // go one char back and clear it
                Key::Backspace => {
                    write!(screen, "{}", cursor::Left(1))?;
                    write!(screen, "{}", clear::AfterCursor)?;
                    line.pop();
                },

                // exit 'for' loop
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
