#![no_std]
#![no_main]
#![allow(unused)]

#[macro_use]
extern crate Mstd;
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

mod alloctest;
mod attrtest;
mod dirtest;
mod forktest;
mod linktest;
mod thread_create;
mod mmmap;
mod pipe;
mod seek;
mod stat;
mod timetest;

#[no_mangle]
fn main(_argc: usize, argv: Vec<String>) -> isize {
    argv.iter().for_each(|test_name| {
        println!("run test {}", test_name);
        match test_name.as_str() {
            "help" => {
                println!("test list:");
                println!("alloc_test");
                println!("attr_test[1-2]");
                println!("fork_test");
                println!("link_test");
                println!("mmap_test");
                println!("pipe_test[1-2]");
                println!("seek_test");
                println!("stat_test");
                println!("dir_test");
                println!("time_test");
                println!("thread_test1");
            }
            "time_test" => {
                timetest::time_test();
            }
            "dir_test" => {
                dirtest::dir_test();
            }
            "alloc_test" => {
                alloctest::alloc_test();
            }
            "attr_test1" => {
                attrtest::attr_test1();
            }
            "attr_test2" => {
                attrtest::attr_test2();
            }
            "fork_test" => {
                forktest::fork_test();
            }
            "link_test" => {
                linktest::link_test();
            }
            "mmap_test" => {
                mmmap::mmap_test();
            }
            "pipe_test1" => {
                pipe::pipe_test1();
            }
            "pipe_test2" => {
                pipe::pipe_test2();
            }
            "seek_test" => {
                seek::seek_test();
            }
            "stat_test" => {
                stat::stat_test();
            }
            "thread_test1" => {
                thread_create::thread_test1();
            }
            _ => {
                println!("test {} not found", test_name);
            }
        }
    });

    0
}
