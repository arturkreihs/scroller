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

pub struct Scroller {
    screen: Arc<termion::raw::RawTerminal<std::io::Stdout>>,
    sender: mpsc::Sender<String>,
    rows: u16,
}

impl Scroller {
    pub fn new() -> Self {
        let screen = Arc::new(std::io::stdout().into_raw_mode().unwrap());
        let (_, rows) = termion::terminal_size().unwrap();
        
        // prepare screen
        {
            let mut screen = screen.lock();
            write!(screen, "{}", clear::All).unwrap();
            write!(screen, "{}", cursor::Goto(1, rows)).unwrap();
            screen.flush().unwrap();
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

        Scroller {
            screen,
            sender: tx,
            rows,
        }
    }

    pub fn write(&mut self, line: &str) {
        self.sender.send(line.into()).unwrap();
    }

    pub fn read(&mut self) -> Option<String> {
        // char buffer
        let mut line: std::vec::Vec<char> = vec![];
        for key in stdin().keys() {
            // take stdout here
            let mut screen = self.screen.lock();

            match key.unwrap() {
                // clear line and do action on enter
                Key::Char('\n') => {
                    write!(screen, "{}", cursor::Goto(1, self.rows)).unwrap();
                    write!(screen, "{}", clear::CurrentLine).unwrap();
                    // self.sender.send(line.iter().collect()).unwrap();
                    // line.clear();
                    return Some(line.iter().collect());
                },

                // add char to buffer and print
                Key::Char(c) => {
                    write!(screen, "{}", c).unwrap();
                    line.push(c);
                },

                // go one char back and clear it
                Key::Backspace => {
                    write!(screen, "{}", cursor::Left(1)).unwrap();
                    write!(screen, "{}", clear::AfterCursor).unwrap();
                    line.pop();
                },

                // exit 'for' loop
                Key::Ctrl('c') => {
                    break;
                }

                // anything else will continue
                _ => continue,
            }
            screen.flush().unwrap();
        }
        None
    }
}

impl Drop for Scroller {
    fn drop(&mut self) {
        self.screen.suspend_raw_mode().unwrap();
    }
}
