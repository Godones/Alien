fn main() {
    let x = 5;
    let p1 = &x as *const _ as *const i32;
    let p2 = &&x as *const _ as *const i32;
    println!("p1 = {:p}", p1);
    println!("p2 = {:p}", p2);
}
