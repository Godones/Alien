#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use Mstd::dbfs::{create_global_bucket, show_dbfs, *};
use Mstd::println;

#[no_mangle]
fn main() -> isize {
    println!("try test dbfs.......");
    let res = create_global_bucket("test");
    println!("create bucket res:{}", res);
    // let res = execute_user_func("test_bucket:test_key", user_func as *const (), &mut [0; 10]);
    // println!("execute user func res:{}", res);
    test_execute_operate();
    0
}

fn test_execute_operate() {
    let addkey_operate = AddKeyOperate::new()
        .add_key("name", b"hello".to_vec())
        .add_key("data1", b"world".to_vec())
        .add_key("data2", b"world".to_vec());

    let buf = [0u8; 20];
    let read_operate = ReadOperate::new()
        .add_key("name")
        .add_key("data1")
        .add_key("data2")
        .set_buf(buf.as_ptr() as usize, 20);

    let mut add_bucket = AddBucketOperate::new("dir1", None);

    let add_operate = AddKeyOperate::new()
        .add_key("uid", b"111".to_vec())
        .add_key("gid", b"222".to_vec())
        .add_key("mode", b"333".to_vec());

    let add_bucket1 = AddBucketOperate::new("dir2", None);
    let operate_set = OperateSet::new()
        .add_operate(Operate::AddKey(add_operate))
        .add_operate(Operate::AddBucket(add_bucket1));
    add_bucket.add_other(Box::new(operate_set));

    let operate_set = OperateSet::new()
        .add_operate(Operate::AddKey(addkey_operate))
        .add_operate(Operate::AddBucket(add_bucket))
        .add_operate(Operate::Read(read_operate));

    dbfs_execute_operate("test", operate_set);
    println!("buf:{:?}", core::str::from_utf8(&buf).unwrap());
    show_dbfs();

    // test step_into rename and delete
    let rename_operate = RenameKeyOperate::new("dir1", "dir2");
    let mut step_into_operate = StepIntoOperate::new("dir2", None);
    let delete_operate = DeleteKeyOperate::new().add_key("mode");

    step_into_operate.add_other(Box::new(
        OperateSet::new().add_operate(Operate::DeleteKey(delete_operate)),
    ));
    let operate_set = OperateSet::new()
        .add_operate(Operate::RenameKey(rename_operate))
        .add_operate(Operate::StepInto(step_into_operate));
    dbfs_execute_operate("test", operate_set);
    println!("----------------------------");
    show_dbfs();
}

fn user_func<'a>(key: &'a str, para: MyPara<'a, 'a>, buf: &mut [u8]) -> isize {
    match para.0 {
        Para::Data(_) => {}
        Para::Bucket(bucket) => {
            let bucket = bucket.get_or_create_bucket(key).unwrap();
            bucket.put("author", "Godones").unwrap();
        }
    }
    0
}
