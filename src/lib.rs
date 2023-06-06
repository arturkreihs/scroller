use std::io::Write;
use std::io::stdin;
use termion::input::TermRead;
use termion::{
    clear,
    cursor,
};
use termion::event::Key;
use termion::raw::IntoRawMode;

use std::thread;
use std::sync::Arc;
use std::sync::mpsc;

pub struct Scroller {
    stdout: Arc<termion::raw::RawTerminal<std::io::Stdout>>,
    tx: mpsc::Sender<String>,
    rows: u16,
}

impl Scroller {
    pub fn new() -> Self {
        let stdout = Arc::new(std::io::stdout().into_raw_mode().unwrap());
        let (_, rows) = termion::terminal_size().unwrap();
        
        // prepare screen
        {
            let mut stdout = stdout.lock();
            write!(stdout, "{}", clear::All).unwrap();
            write!(stdout, "{}", cursor::Goto(1, rows)).unwrap();
            stdout.flush().unwrap();
        }

        let (tx, rx) = mpsc::channel::<String>();

        // start printing
        {
            let stdout = Arc::clone(&stdout);
            thread::spawn(move || {
                loop {
                    // wait for text to arrive
                    let line = rx.recv().unwrap();

                    // take stdout here
                    let mut stdout = stdout.lock();

                    // save current position
                    write!(stdout, "{}", termion::cursor::Save).unwrap();

                    // scroll up
                    write!(stdout, "{}", termion::scroll::Up(1)).unwrap();

                    // go to bottom line - 1
                    write!(stdout, "{}", cursor::Goto(1, rows - 1)).unwrap();

                    // write text to that line
                    write!(stdout, "{}", line).unwrap();
                    
                    // restore position
                    write!(stdout, "{}", termion::cursor::Restore).unwrap();

                    stdout.flush().unwrap();
                }
            });
        }

        Scroller {
            stdout,
            tx,
            rows,
        }
    }

    pub fn pool(&mut self) {
        // char buffer
        let mut line: std::vec::Vec<char> = vec![];
        for key in stdin().keys() {
            // take stdout here
            let mut stdout = self.stdout.lock();

            match key.unwrap() {
                // clear line and do action on enter
                Key::Char('\n') => {
                    write!(stdout, "{}", cursor::Goto(1, self.rows)).unwrap();
                    write!(stdout, "{}", clear::CurrentLine).unwrap();
                    self.tx.send(line.iter().collect()).unwrap();
                    line.clear();
                },

                // add char to buffer and print
                Key::Char(c) => {
                    write!(stdout, "{}", c).unwrap();
                    line.push(c);
                },

                // go one char back and clear it
                Key::Backspace => {
                    write!(stdout, "{}", cursor::Left(1)).unwrap();
                    write!(stdout, "{}", clear::AfterCursor).unwrap();
                    line.pop();
                },

                // exit 'for' loop
                Key::Ctrl('c') => {
                    break;
                }

                // anything else will continue
                _ => continue,
            }
            stdout.flush().unwrap();
        }
    }

    pub fn exit(self) {
        self.stdout.suspend_raw_mode().unwrap();
    }
}
