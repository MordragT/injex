use {injector::prelude::*, std::thread};

#[allow(dead_code)]
#[link_section = ".init_array"]
static INITIALIZE: fn() = init;

fn init() {
    thread::spawn(move || -> thread::Result<()> {
        let manipulator = InternalManipulator {};
        println!("{:?}", manipulator.memory_maps());
        let addr = match manipulator.find(0, 1024, &[0, 0, 8, 16, 32]) {
            Some(addr) => addr,
            None => panic!("Signature not found"),
        };
        manipulator.write(addr, &[255, 255, 255, 0]).unwrap();
        Ok(())
    });
}

const NUM_GUNS: usize = 10;

#[repr(C)]
struct PlayerState {
    health: i32,
    armour: i32,
    lastspawn: i32,
    lastregen: i32,
    primary: i32,
    secondary: i32,
    perk: [i32; 2],
    nextprimary: i32,
    nextsecondary: i32,
    nextperk: [i32; 2],
    gunselect: i32,
    akimbo: i32,
    scoping: i32,
    ammo: [i32; NUM_GUNS],
    mag: [i32; NUM_GUNS],
    gunwait: [i32; NUM_GUNS],
    pstatshots: [i32; NUM_GUNS],
    pstatdamage: [i32; NUM_GUNS],
}

impl DefaultInit for PlayerState {
    fn is_default_init(&self) -> bool {
        self.health == 100 && self.armour == 0
    }
}
