# Scroller
A library that changes terminal into one scrolling part for output and the other part, bottom line, for input. It is written to be able to work asynchronously.
### Usage
```
use anyhow::Result;
use scroller::Scroller;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let scr = Arc::new(Scroller::new()?);

    {
        let scr = Arc::clone(&scr);
        tokio::spawn(async move {
            let mut cntr = 0;
            loop {
                scr.write(&format!("line {}", cntr)).unwrap();
                cntr += 1;
                tokio::time::sleep(core::time::Duration::from_millis(1000)).await;
            }
        });
    }

    while let Some(input) = scr.read()? {
        let mut s = String::from(&input);
        s.insert_str(0, ">>> ");
        s.push_str(" <<<");
        scr.write(&s)?;
    }

    Ok(())
}
```
