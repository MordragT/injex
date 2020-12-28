use injector::prelude::*;

fn main() {
    let process = AnonManipulator::new("").unwrap();
    println!(
        "Assaultcube Reloaded process found succesfully: {}",
        process.pid()
    );
    inject(
        &process,
        process.pid(),
        "/home/tom/Git/injector/examples/internal/target/debug/libinternal.so",
    )
    .unwrap();
}
