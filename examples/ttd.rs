use injector::prelude::*;

fn main() {
    let process = AnonManipulator::new("openttd").unwrap();
    println!("Process found succesfully: {}", process.pid());
    internal(process);
}

fn internal<T: Injection>(process: T) {
    process
        .inject("/home/tom/Git/injector/examples/internal/target/debug/libinternal.so")
        .unwrap();
}

fn external<T: MemoryManipulation>(process: T) {
    process
        .write(0x7f64944f8b20, &[0xff, 0xff, 0xff, 0xfe])
        .unwrap();
    let mut buf = [0_u8; 4];
    process.read(0x7f64944f8b20, &mut buf).unwrap();
    println!("{:?}", buf);
    let result = process.read_structure::<u32>(0x7f64944f8b20).unwrap();
    println!("{}", result);
    // loop {
    //     process
    //         .write_structure::<u32>(0x7f64944f8b20, 10000000)
    //         .unwrap();
    // }
    let signature = process
        .find(
            0x556c29ea1000,
            0x556c39ea1000,
            &[0x60, 0xda, 0xab, 0x2c, 0x6c, 0x55, 0x00, 0x00, 0xd0],
        )
        .unwrap();
    println!("{:x}", signature);

    let wildcard = process
        .find_wildcard(0x556c29ea1000, 0x556c39ea1000, "60 da ab 2c 6c 55 :: :: d0")
        .unwrap();
    println!("{:x}", wildcard);
}
