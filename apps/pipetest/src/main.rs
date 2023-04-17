#![no_std]
#![no_main]


use Mstd::fs::{close, read, write};
use Mstd::ipc::pipe;
use Mstd::println;
use Mstd::process::{fork, wait};

static STR: &str = "Hello, world!";

#[no_mangle]
pub fn main() -> i32 {
    // create pipe
    let mut pipe_fd = [0u32; 2];
    pipe(&mut pipe_fd);
    let pipe_fd = [pipe_fd[0] as usize, pipe_fd[1] as usize];
    // read end
    assert_eq!(pipe_fd[0], 3);
    // write end
    assert_eq!(pipe_fd[1], 4);
    if fork() == 0 {
        // child process, read from parent
        // close write_end
        close(pipe_fd[1]);
        let mut buffer = [0u8; 32];
        let len_read = read(pipe_fd[0], &mut buffer) as usize;
        // close read_end
        close(pipe_fd[0]);
        assert_eq!(core::str::from_utf8(&buffer[..len_read]).unwrap(), STR);
        println!("Read OK, child process exited!");
        0
    } else {
        // parent process, write to child
        // close read end
        close(pipe_fd[0]);
        assert_eq!(write(pipe_fd[1], STR.as_bytes()), STR.len() as isize);
        // close write end
        close(pipe_fd[1]);
        let mut child_exit_code: i32 = 0;
        wait(&mut child_exit_code);
        assert_eq!(child_exit_code, 0);
        println!("pipetest passed!");
        0
    }
}