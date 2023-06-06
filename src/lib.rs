use std::io::Write;
use std::io::stdin;
use std::thread;
use termion::{
    clear,
    cursor,
    input::TermRead,
    event::Key,
    raw::IntoRawMode,
};
use std::sync::{
    Arc,
    mpsc,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScrollerError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Sender(#[from] std::sync::mpsc::SendError<std::string::String>)
}

pub struct Scroller {
    screen: Arc<termion::raw::RawTerminal<std::io::Stdout>>,
    sender: mpsc::Sender<String>,
    rows: u16,
}

impl Default for Scroller {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Scroller {
    pub fn new() -> Result<Self, ScrollerError> {
        let screen = Arc::new(std::io::stdout().into_raw_mode()?);
        let (_, rows) = termion::terminal_size()?;
        
        // prepare screen
        {
            let mut screen = screen.lock();
            write!(screen, "{}", clear::All)?;
            write!(screen, "{}", cursor::Goto(1, rows))?;
            screen.flush()?;
        }

        let (tx, rx) = mpsc::channel::<String>();

        // start printing
        {
            let screen = Arc::clone(&screen);
            thread::spawn(move || {
                loop {
                    // wait for text to arrive
                    let line = rx.recv().unwrap();

                    // take stdout here
                    let mut screen = screen.lock();

                    // save current position
                    write!(screen, "{}", termion::cursor::Save).unwrap();

                    // scroll up
                    write!(screen, "{}", termion::scroll::Up(1)).unwrap();

                    // go to bottom line - 1
                    write!(screen, "{}", cursor::Goto(1, rows - 1)).unwrap();

                    // write text to that line
                    write!(screen, "{}", line).unwrap();
                    
                    // restore position
                    write!(screen, "{}", termion::cursor::Restore).unwrap();

                    screen.flush().unwrap();
                }
            });
        }

        Ok(Scroller {
            screen,
            sender: tx,
            rows,
        })
    }

    pub fn write(&mut self, line: &str) -> Result<(), ScrollerError> {
        self.sender.send(line.into())?;
        Ok(())
    }

    pub fn read(&mut self) -> Result<Option<String>, ScrollerError> {
        // char buffer
        let mut line: std::vec::Vec<char> = vec![];
        for key in stdin().keys() {
            // take stdout here
            let mut screen = self.screen.lock();

            match key? {
                // clear line and do action on enter
                Key::Char('\n') => {
                    write!(screen, "{}", cursor::Goto(1, self.rows))?;
                    write!(screen, "{}", clear::CurrentLine)?;
                    return Ok(Some(line.iter().collect()));
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
