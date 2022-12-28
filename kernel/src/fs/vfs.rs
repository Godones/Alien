use crate::driver::QEMU_BLOCK_DEVICE;
use crate::print::console::get_line;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::error::Error;
use fat32::{Dir, Fat32};
use fat32_trait::DirectoryLike;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ROOT_DIR: Arc<Dir> = {
        let device = QEMU_BLOCK_DEVICE.lock();
        let device = device.as_ref().unwrap();
        let fs = Fat32::new(device.clone()).unwrap();
        fs.root_dir()
    };
}

pub fn fs_repl() {
    let mut path_record: Vec<String> = Vec::new();
    let mut current_dir: Arc<dyn DirectoryLike<Error: Error + 'static>> = ROOT_DIR.clone();
    loop {
        let mut path = String::new();
        path_record.iter().for_each(|x| {
            path.push_str(x);
            path.push_str("/");
        });
        if path_record.len() != 0 {
            path.pop();
        }
        print!("{}>", path);
        let input = get_line();
        let input = input.split(" ").collect::<Vec<&str>>();
        if input.len() == 0 {
            continue;
        }
        match input[0].as_ref() {
            "pwd" => {
                print!("/");
                path_record.iter().for_each(|x| {
                    print!("{}/", x);
                });
                println!("");
            }
            "ls" => {
                current_dir.list().unwrap().iter().for_each(|x| {
                    println!("{}", x);
                });
            }
            "cd" => {
                if input.len() == 1 {
                    println!("cd: missing operand");
                    continue;
                }
                match input[1] {
                    ".." => {
                        if path_record.len() == 0 {
                            println!("cd: no such file or directory");
                            continue;
                        }
                        path_record.pop();
                        current_dir = ROOT_DIR.clone();
                        path_record.iter().for_each(|x| {
                            current_dir = current_dir.cd(x).unwrap();
                        });
                    }
                    "." => {
                        continue;
                    }
                    other => {
                        let dir = current_dir.cd(other);
                        if dir.is_err() {
                            println!("cd: no such file or directory");
                            continue;
                        }
                        path_record.push(input[1].to_string());
                        current_dir = dir.unwrap();
                    }
                }
            }
            "touch" => {
                if input.len() == 1 {
                    println!("touch: missing operand");
                    continue;
                }
                let file = current_dir.create_file(input[1]);
                if file.is_err() {
                    println!("touch: cannot create file");
                    continue;
                }
            }
            "mkdir" => {
                if input.len() == 1 {
                    println!("mkdir: missing operand");
                    continue;
                }
                let ans = current_dir.create_dir(input[1]);
                if ans.is_err() {
                    println!("mkdir: cannot create directory");
                    continue;
                }
            }
            "cat" => {
                if input.len() == 1 {
                    println!("cat: missing operand");
                    continue;
                }
                let file = current_dir.open(input[1]);
                if file.is_err() {
                    println!("cat: no such file or directory");
                    continue;
                }
                let file = file.unwrap();
                let f_size = file.size();
                let ans = file.read(0, f_size);
                if ans.is_err() {
                    println!("cat: cannot read file");
                    continue;
                }
                let ans = ans.unwrap();
                let ans = String::from_utf8(ans).unwrap();
                println!("{}", ans);
            }
            "rename" => {
                if input.len() != 4 {
                    println!("rename {{old}} {{new}} -d/f: missing operand");
                    continue;
                }
                let ans = match input[3] {
                    "-d" => current_dir.rename_dir(input[1], input[2]),
                    "-f" => current_dir.rename_file(input[1], input[2]),
                    _ => {
                        println!("rename {{old}} {{new}} -d/f: missing operand");
                        continue;
                    }
                };
                if ans.is_err() {
                    println!("rename: cannot rename");
                    continue;
                }
            }
            "rm" => {
                if input.len() != 3 {
                    println!("rm {{name}} -d/f: missing operand");
                    continue;
                }
                let ans = match input[2] {
                    "-d" => current_dir.delete_dir(input[1]),
                    "-f" => current_dir.delete_file(input[1]),
                    _ => {
                        println!("rm {{name}} -d/f: missing operand");
                        continue;
                    }
                };
                if ans.is_err() {
                    println!("rm: cannot remove");
                    continue;
                }
            }
            _ => {}
        }
    }
}
