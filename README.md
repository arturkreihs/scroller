# Scroller
### Usage
```
use anyhow::Result;
use scroller::Scroller;

fn main() -> Result<()> {
    let mut scr = Scroller::new()?;

    loop {
        match scr.read()? {
            Some(s) => scr.write(&s)?,
            None => break,
        }
    }
    Ok(())
}
```
