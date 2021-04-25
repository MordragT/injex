## Injex

- aims to provide a library in rust to alter other processes
- at the moment only linux is supported, if i enter a state where i am satisfied with the linux implementation i will look at OpenBSD/Mac/Windows

#### Links

- [crate](https://crates.io/crates/injex)
- [documentation](https://docs.rs/injex/0.1.0/injex/)

#### Example

```rust
use injex::prelude::*;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let anon = AnonManipulator::new("game_name")?;
    inject(&anon, anon.pid(), "path/to/dynamic_library")?;
    Ok(())
}
```
```rust
// Dynamic Library in its own crate

use std::thread;
use injex::prelude::*;

#[link_section = ".init_array"]
static INITIALIZE: fn() = init;

fn init() {
   thread::spawn(move || -> thread::Result<()> {
       let manipulator = InternalManipulator {}
       println!("{:?}", manipulator.memory_maps());
       let address = manipulator.find(0, 1024, &[0, 3, 10, 32, 1]).unwrap();
       loop {
           manipulator.write(address, &[255, 255, 255, 255]).unwrap();
       }
   });
}
```

#### Credit

- my injection function is basically a rewrite of [dlinject](https://github.com/DavidBuchanan314/dlinject) in rust

#### LICENSE

- MIT