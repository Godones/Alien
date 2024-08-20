# static keys 

通过修改代码来实现不同的分支，而不是使用原子变量进行判断。

核心机制是rust的`naked function` 允许我们在函数中包含单个汇编代码块。

## Example
```rust
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

```