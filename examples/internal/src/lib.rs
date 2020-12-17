use {injector::prelude::*, std::thread};

#[allow(dead_code)]
#[link_section = ".init_array"]
static INITIALIZE: fn() = init;

fn init() {
    thread::spawn(move || {
        let manipulator = InternalManipulator {};
        println!("{:?}", manipulator.memory_maps());
    });
}
