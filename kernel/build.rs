use std::fs::File;
use std::io::Write;
use std::path::Path;
// use syscall_table::scan::scan_and_generate;

fn main() {
    println!("cargo:rerun-if-changed={}", "src/");
    let path = Path::new("src/trace/kernel_symbol.S");
    if !path.exists() {
        let mut file = File::create(path).unwrap();
        write!(file, ".section .rodata\n").unwrap();
        write!(file, ".align 3\n").unwrap();
        write!(file, ".global symbol_num\n").unwrap();
        write!(file, ".global symbol_address\n").unwrap();
        write!(file, ".global symbol_index\n").unwrap();
        write!(file, ".global symbol_name\n").unwrap();
        write!(file, "symbol_num:\n").unwrap();
        write!(file, ".quad {}\n", 0).unwrap();
        write!(file, "symbol_address:\n").unwrap();
        write!(file, "symbol_index:\n").unwrap();
        write!(file, "symbol_name:\n").unwrap();
    }
    scan_and_generate("src/syscall.rs".to_string());
}
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::path::PathBuf;
use std::string::{String, ToString};
use std::vec::Vec;
use std::{format, fs};

pub fn scan_and_generate(path: String) {
    // read all files in the directory rescursively
    let mut target_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap();
    let mut context = Vec::new();
    let import = b"\
    use spin::Once;\n\
    use syscall_table::{register_syscall, Table};\n\
    static SYSCALL_TABLE: Once<Table> = Once::new();\n\
    pub fn register_all_syscall(){\n\
        \tlet mut table = Table::new();\n\
        \tregister_syscall!(table,\n\
    ";
    context.extend_from_slice(import);

    let mut import = HashSet::new();
    scan(&mut import, &mut context, PathBuf::from("src"));
    let end = b"\n\
        \t);\n\
        \tSYSCALL_TABLE.call_once(||table);\n\
    }\n\
    pub fn do_syscall(id:usize,args:&[usize])->isize{\n\
        \tlet res = SYSCALL_TABLE.get().unwrap().do_call(id,&args);\n\
        \tif res.is_none(){\n\
        \t\t    return -1;\n\
        \t}else {\n\
        \t    return res.unwrap();\n\
        \t}\n\
    }\n\
    ";
    context.extend_from_slice(end);
    import.iter().for_each(|m| {
        let import = format!("use {};\n", m);
        context.extend_from_slice(import.as_bytes());
    });
    target_file.write_all(&context).unwrap();
}

fn scan(import: &mut HashSet<String>, context: &mut Vec<u8>, dir: PathBuf) {
    let entries = fs::read_dir(dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            scan(import, context, path);
        } else {
            if path.extension().unwrap() == "rs" {
                let file = File::open(path.clone()).unwrap();
                let mut buf_reader = std::io::BufReader::new(file);
                let mut line = String::new();
                while buf_reader.read_line(&mut line).unwrap() > 0 {
                    if line.contains("#[syscall_func") {
                        // #[syscall_func(1)]
                        // find the id from the line
                        let id = line.split("(").nth(1).unwrap().split(")").next().unwrap();
                        let id = id.parse::<usize>().unwrap();
                        // find the function name
                        let mut func_name = String::new();
                        if buf_reader.read_line(&mut func_name).unwrap() > 0 {
                            if func_name.contains("pub fn") {
                                func_name = func_name
                                    .split("pub fn")
                                    .nth(1)
                                    .unwrap()
                                    .split("(")
                                    .next()
                                    .unwrap()
                                    .to_string();
                            } else {
                                panic!("error: the function should be public");
                            }
                        } else {
                            panic!("error: the function name is not found");
                        }
                        // find the mod according the path
                        let mut mod_name = String::from("crate");
                        let path = path.to_str().unwrap().to_string();
                        let component = path.split("/").collect::<Vec<&str>>();
                        let correct = if component.len() == 2 {
                            component.len()
                        } else {
                            component.len() - 1
                        };
                        for i in 0..correct {
                            if component[i] == "src" {
                            } else {
                                mod_name.push_str("::");
                                if component[i].ends_with(".rs") {
                                    mod_name.push_str(component[i].strip_suffix(".rs").unwrap());
                                } else {
                                    mod_name.push_str(component[i]);
                                }
                            }
                        }
                        mod_name.push_str("::");
                        mod_name.push_str(&func_name.trim());
                        import.insert(mod_name);
                        // generate the code
                        let code = format!("\t({},{}),\n", id, func_name);
                        context.extend_from_slice(code.as_bytes());
                    }
                    line.clear();
                }
            }
        }
    }
}
