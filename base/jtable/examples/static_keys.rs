use jtable::*;
use paste::*;
define_static_key_true!(TRUE_TEST);

fn main() {
    test_static_key();
}

define_static_key_true!(TRUE_MASK);
define_static_key_false!(FALSE_MASK);

pub fn test_static_key() {
    if static_branch_likely!(TRUE_MASK) {
        println!("static_branch_likely");
    }
    // need to modify the text section to disable the static key
    // static_branch_disable!(TRUE_MASK);

    if static_branch_unlikely!(FALSE_MASK) {
        println!("static_branch_unlikely");
    }
    // need to modify the text section to enable the static key
    // static_branch_enable!(FALSE_MASK);
}
