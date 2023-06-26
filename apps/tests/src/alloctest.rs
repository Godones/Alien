use alloc::vec::Vec;

pub fn alloc_test() {
    let mut v = Vec::new();
    for i in 0..2000 {
        v.push(i);
    }
    v.clear();
    println!("Alloc test success!");
}