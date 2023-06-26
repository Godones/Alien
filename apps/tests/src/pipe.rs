use alloc::format;

use Mstd::fs::{close, read, write};
use Mstd::ipc::pipe;
use Mstd::process::{fork, wait};

const STR: &str = "Hello, world!";

pub fn pipe_test1() -> isize {
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


const LENGTH: usize = 3000;

pub fn pipe_test2() -> isize {
    // create pipes
    // parent write to child
    let mut down_pipe_fd = [0u32; 2];
    // child write to parent
    let mut up_pipe_fd = [0u32; 2];
    pipe(&mut down_pipe_fd);
    pipe(&mut up_pipe_fd);
    let down_pipe_fd = [down_pipe_fd[0] as usize, down_pipe_fd[1] as usize];
    let up_pipe_fd = [up_pipe_fd[0] as usize, up_pipe_fd[1] as usize];

    let mut random_str = [0u8; LENGTH];
    if fork() == 0 {
        // close write end of down pipe
        close(down_pipe_fd[1]);
        // close read end of up pipe
        close(up_pipe_fd[0]);
        assert_eq!(read(down_pipe_fd[0], &mut random_str) as usize, LENGTH);
        close(down_pipe_fd[0]);
        let sum: usize = random_str.iter().map(|v| *v as usize).sum::<usize>();
        println!("sum = {}(child)", sum);
        let sum_str = format!("{}", sum);
        write(up_pipe_fd[1], sum_str.as_bytes());
        close(up_pipe_fd[1]);
        println!("Child process exited!");
        0
    } else {
        // close read end of down pipe
        close(down_pipe_fd[0]);
        // close write end of up pipe
        close(up_pipe_fd[1]);
        // generate a long random string
        for ch in random_str.iter_mut() {
            *ch = 1 as u8;
        }
        // send it
        assert_eq!(
            write(down_pipe_fd[1], &random_str) as usize,
            random_str.len()
        );
        // close write end of down pipe
        close(down_pipe_fd[1]);
        // calculate sum(parent)
        let sum: usize = random_str.iter().map(|v| *v as usize).sum::<usize>();
        println!("sum = {}(parent)", sum);
        // recv sum(child)
        let mut child_result = [0u8; 32];
        let result_len = read(up_pipe_fd[0], &mut child_result) as usize;
        close(up_pipe_fd[0]);
        // check
        assert_eq!(
            sum,
            str::parse::<usize>(core::str::from_utf8(&child_result[..result_len]).unwrap())
                .unwrap()
        );
        let mut _unused: i32 = 0;
        wait(&mut _unused);
        println!("pipe_large_test passed!");
        0
    }
}