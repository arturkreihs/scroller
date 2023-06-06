# scroller
### Usage
```
use scroller::Scroller;

fn main() {
    let mut scr = Scroller::new();

    loop {
        match scr.read() {
            Some(s) => scr.write(&s),
            None => break,
        }
    }
}
```
