use std::time::Instant;

fn main() {
    let now = Instant::now();
    println!("now: {:#x?}", now);
    println!("Hello, world!");
    let thread = std::thread::spawn(|| {
        println!("Hello from thread!");
    });
    thread.join().unwrap();
}
