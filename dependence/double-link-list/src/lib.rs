#![allow(deref_nullptr)]
#![feature(core_intrinsics)]
#![no_std]
//! implement list_head in linux
use core::default::Default;
use core::iter::Iterator;

/// ListHead define
#[derive(Debug, Copy, Clone)]
pub struct ListHead {
    pub prev: *mut ListHead,
    pub next: *mut ListHead,
}

impl Default for ListHead {
    fn default() -> Self {
        Self {
            prev: core::ptr::null_mut(),
            next: core::ptr::null_mut(),
        }
    }
}
impl ListHead {
    pub const fn new() -> Self {
        Self {
            prev: core::ptr::null_mut(),
            next: core::ptr::null_mut(),
        }
    }
}

#[macro_export]
macro_rules! list_head_init {
    ($ident:expr) => {
        $ident.prev = &$ident as *const _ as *mut _;
        $ident.next = &$ident as *const _ as *mut _;
    };
}

#[macro_export]
macro_rules! list_head {
    ($ident:ident) => {
        let mut $ident = ListHead::default();
        list_head_init!($ident);
    };
}

/// usage
///```
/// use double_link_list::{list_add, list_head, list_head_init,list_add_in, ListHead};
/// use core::ptr::{write_volatile, read_volatile};
/// struct Demo {
///    pub list_head: ListHead,
///    first:usize,
/// }
/// list_head!(demo);
/// let mut demo1 = Demo {
///    list_head: {
///        let mut list_head = ListHead::default();
///        list_head_init!(list_head);
///        list_head
///    },
///    first: 1,
/// };
/// list_add!(&demo1.list_head as *const _ as *mut ListHead, &demo as *const _ as *mut ListHead);
///
///```
///
#[macro_export]
macro_rules! list_add {
    ($new:expr,$head:expr) => {
        list_add_in!($new, $head, (*$head).next);
    };
}

#[macro_export]
macro_rules! list_add_tail {
    ($new:expr,$head:expr) => {
        list_add_in_tail!($new, (*$head).prev, $head);
    };
}

#[macro_export]
macro_rules! list_add_in_tail {
    ($new:expr,$prev:expr,$next:expr) => {
        // println!("new:{:?}, prev: {:?}, next: {:?}", $new,$prev, $next);
        // use volatile to read and write
        unsafe {
            (*$prev).next = $new;
            (*$new).next = $next;
            (*$new).prev = $prev;
            (*$next).prev = $new;
        }
        // 这里不能使用如下的形式
        // (*$new).next = $next;
        // (*$new).prev = $prev;
        // (*$prev).next = $new;
        // (*$next).prev = $new;
        // 原因是可能后面两句会被优化
    };
}
#[macro_export]
macro_rules! list_add_in {
    ($new:expr,$prev:expr,$next:expr) => {
        // println!("new:{:?}, prev: {:?}, next: {:?}", $new,$prev, $next);
        // use volatile to read and write
        unsafe {
            (*$next).prev = $new;
            (*$new).next = $next;
            (*$new).prev = $prev;
            (*$prev).next = $new;
            // core::intrinsics::volatile_store((*$new).next, core::intrinsics::volatile_load($next));
            // core::intrinsics::volatile_store((*$new).prev, core::intrinsics::volatile_load($prev));
            // core::intrinsics::volatile_store((*$prev).next, core::intrinsics::volatile_load($new));
            // core::intrinsics::volatile_store((*$next).prev, core::intrinsics::volatile_load($new));
        }
        // 这里不能使用如下的形式
        // (*$new).next = $next;
        // (*$new).prev = $prev;
        // (*$prev).next = $new;
        // (*$next).prev = $new;
        // 原因是可能后面两句会被优化
    };
}
#[macro_export]
macro_rules! list_del_check {
    ($entry:expr) => {
        unsafe {
            if (*$entry).next == $entry || (*$entry).prev == $entry {
                false
            } else {
                true
            }
        }
    };
}

#[macro_export]
macro_rules! is_list_empty {
    ($entry:expr) => {
        !list_del_check!($entry)
    };
}

#[macro_export]
macro_rules! list_del {
    ($entry:expr) => {
        if (!list_del_check!($entry)) {
            panic!("list_del_check failed");
        }
        unsafe {
            (*(*$entry).next).prev = (*$entry).prev;
            (*(*$entry).prev).next = (*$entry).next;
            *$entry = ListHead {
                prev: core::ptr::null_mut(),
                next: core::ptr::null_mut(),
            };
        }
    };
}

#[macro_export]
macro_rules! offset_of {
    ($type:ty,$field:ident) => {
        unsafe { &(*(core::ptr::null::<$type>())).$field as *const _ as usize }
    };
}

#[macro_export]
macro_rules! container_of {
    ($ptr:expr,$type:ty,$member:ident) => {
        ($ptr - offset_of!($type, $member)) as *mut $type
    };
}

#[macro_export]
macro_rules! to_list_head_ptr {
    ($expr:expr) => {
        &$expr as *const _ as *mut ListHead
    };
}

#[macro_export]
macro_rules! align_to {
    ($addr:expr, $align:expr) => {
        ($addr + $align - 1) & !($align - 1)
    };
}

impl ListHead {
    pub fn iter(&self) -> Iter {
        Iter {
            head: self,
            cur: self.next,
        }
    }
    pub fn len(&self) -> usize {
        let mut len = 0;
        for _ in self.iter() {
            len += 1;
        }
        len
    }
}

pub struct Iter<'a> {
    head: &'a ListHead,
    cur: *mut ListHead,
}

impl Iterator for Iter<'_> {
    type Item = *mut ListHead;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == to_list_head_ptr!(*self.head) {
            None
        } else {
            let ret = self.cur;
            unsafe {
                self.cur = (*self.cur).next;
            }
            Some(ret)
        }
    }
}

#[cfg(test)]
mod test {
    use super::ListHead;
    use super::{list_head, list_head_init};
    #[allow(unused)]
    #[derive(Debug)]
    struct Demo {
        list_head: ListHead,
        first: usize,
        second: usize,
    }

    #[test]
    fn test_list_head_add() {
        list_head!(head);
        let mut demo1 = Demo {
            list_head: ListHead::default(),
            first: 1,
            second: 2,
        };
        list_head_init!(demo1.list_head);
        assert_eq!(to_list_head_ptr!(head), head.prev);
        assert_eq!(head.next, head.prev);

        assert_eq!(demo1.list_head.prev, demo1.list_head.next);
        assert_eq!(to_list_head_ptr!(demo1.list_head), demo1.list_head.prev);

        // head <--> demo1
        list_add!(to_list_head_ptr!(demo1.list_head), to_list_head_ptr!(head));

        assert_eq!(head.next, to_list_head_ptr!(demo1.list_head));
        assert_eq!(head.prev, to_list_head_ptr!(demo1.list_head));

        let mut demo2 = Demo {
            list_head: ListHead::default(),
            first: 1,
            second: 2,
        };
        list_head_init!(demo2.list_head);
        list_add!(to_list_head_ptr!(demo2.list_head), to_list_head_ptr!(head));
        // head <--> demo2 <--> demo1
        assert_eq!(head.next, to_list_head_ptr!(demo2.list_head));
        assert_eq!(head.prev, to_list_head_ptr!(demo1.list_head));
        assert_eq!(demo1.list_head.prev, to_list_head_ptr!(demo2.list_head));
        assert_eq!(demo2.list_head.next, to_list_head_ptr!(demo1.list_head));
        assert_eq!(demo1.list_head.next, to_list_head_ptr!(head));
        assert_eq!(demo2.list_head.prev, to_list_head_ptr!(head));

        println!("head.next:{:?}", head.next);
        println!("head.prev:{:?}", head.prev);
        println!("demo1_list_head:{:?}", to_list_head_ptr!(demo1.list_head));
        println!("demo1.prev:{:?}", demo1.list_head.prev);
        println!("demo1.next:{:?}", demo1.list_head.next);
    }
    #[test]
    fn test_list_head_add_tail() {
        list_head!(head);
        let mut demo1 = Demo {
            list_head: ListHead::default(),
            first: 1,
            second: 2,
        };
        list_head_init!(demo1.list_head);
        assert_eq!(to_list_head_ptr!(head), head.prev);
        assert_eq!(head.next, head.prev);

        assert_eq!(demo1.list_head.prev, demo1.list_head.next);
        assert_eq!(to_list_head_ptr!(demo1.list_head), demo1.list_head.prev);

        // head <--> demo1
        list_add_tail!(to_list_head_ptr!(demo1.list_head), to_list_head_ptr!(head));

        assert_eq!(head.next, to_list_head_ptr!(demo1.list_head));
        assert_eq!(head.prev, to_list_head_ptr!(demo1.list_head));

        let mut demo2 = Demo {
            list_head: ListHead::default(),
            first: 1,
            second: 2,
        };
        list_head_init!(demo2.list_head);
        list_add_tail!(to_list_head_ptr!(demo2.list_head), to_list_head_ptr!(head));
        // head <--> demo1 <--> demo2
        assert_eq!(head.next, to_list_head_ptr!(demo1.list_head));
        assert_eq!(head.prev, to_list_head_ptr!(demo2.list_head));
        assert_eq!(demo1.list_head.prev, to_list_head_ptr!(head));
        assert_eq!(demo2.list_head.next, to_list_head_ptr!(head));
        assert_eq!(demo1.list_head.next, to_list_head_ptr!(demo2.list_head));
        assert_eq!(demo2.list_head.prev, to_list_head_ptr!(demo1.list_head));

        println!("head.next:{:?}", head.next);
        println!("head.prev:{:?}", head.prev);
        println!("demo1_list_head:{:?}", to_list_head_ptr!(demo1.list_head));
        println!("demo1.prev:{:?}", demo1.list_head.prev);
        println!("demo1.next:{:?}", demo1.list_head.next);
    }
    #[test]
    fn test_list_del() {
        list_head!(head);
        let mut demo1 = Demo {
            list_head: ListHead::default(),
            first: 1,
            second: 2,
        };
        list_head_init!(demo1.list_head);
        // head <--> demo1
        list_add!(to_list_head_ptr!(demo1.list_head), to_list_head_ptr!(head));
        let mut demo2 = Demo {
            list_head: ListHead::default(),
            first: 1,
            second: 2,
        };
        list_head_init!(demo2.list_head);
        list_add!(to_list_head_ptr!(demo2.list_head), to_list_head_ptr!(head));
        // head <--> demo2 <--> demo1
        list_del!(to_list_head_ptr!(demo2.list_head));
        // head <--> demo1
        assert_eq!(head.next, to_list_head_ptr!(demo1.list_head));
        assert_eq!(head.prev, to_list_head_ptr!(demo1.list_head));
        assert_eq!(demo1.list_head.next, to_list_head_ptr!(head));
        assert_eq!(demo1.list_head.prev, to_list_head_ptr!(head));

        list_del!(to_list_head_ptr!(demo1.list_head));
        // head
        assert_eq!(head.next, to_list_head_ptr!(head));
        assert_eq!(head.prev, to_list_head_ptr!(head));

        println!("head.next:{:?}", head.next);
        println!("head.prev:{:?}", head.prev);
        println!("demo1_list_head:{:?}", to_list_head_ptr!(demo1.list_head));
        println!("demo1.prev:{:?}", demo1.list_head.prev);
        println!("demo1.next:{:?}", demo1.list_head.next);
    }
    #[test]
    fn test_offset_of() {
        let mut demo1 = Demo {
            list_head: ListHead::default(),
            first: 1,
            second: 2,
        };
        let list_head_ptr = to_list_head_ptr!(demo1.list_head);
        let list_head_offset = offset_of!(Demo, list_head);
        let list_head_ptr2 = list_head_ptr as usize - list_head_offset;
        let demo1_ptr = list_head_ptr2 as *mut Demo;
        assert_eq!(demo1_ptr, &mut demo1 as *mut Demo);
    }
    #[test]
    fn test_container_of() {
        let mut demo1 = Demo {
            list_head: ListHead::default(),
            first: 1,
            second: 2,
        };
        let list_head_ptr = to_list_head_ptr!(demo1.list_head);
        let demo1_ptr = container_of!(list_head_ptr as usize, Demo, list_head);
        assert_eq!(demo1_ptr, &mut demo1 as *mut Demo);
    }
    #[test]
    fn test_align_to() {
        assert_eq!(align_to!(1, 8), 8);
        assert_eq!(align_to!(8, 8), 8);
        assert_eq!(align_to!(9, 8), 16);
        assert_eq!(align_to!(16, 8), 16);
        assert_eq!(align_to!(17, 8), 24);
    }

    #[test]
    fn test_list_head_iter() {
        list_head!(head); //链表头
        list_head!(head2); //链表头
        list_head!(head3); //链表头
        list_add_tail!(to_list_head_ptr!(head2), to_list_head_ptr!(head));
        list_add_tail!(to_list_head_ptr!(head3), to_list_head_ptr!(head));
        head.iter().for_each(|list_head| {
            println!("list_head:{:?}", list_head);
        });
        assert_eq!(head.next, to_list_head_ptr!(head2));
        assert_eq!(head.len(), 2);
    }
}
