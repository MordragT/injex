use injector::prelude::*;

#[link_section = ".init_array"]
static INITIALIZE: fn() = init;

fn init() {
    println!("Injected !")
}
