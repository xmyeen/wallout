use std::env;
use libloading;

/**
 *  如果使用c的数结构，参考此页。
 *  https://static.rust-lang.org/doc/master/book/ffi.html
 * 
 */


extern "C" fn callback() -> i32 {
    30i32
}

type NoBack = fn();
type AddFunc = fn(isize, isize) -> isize;

fn exec_none(library_path: &str) {
    unsafe {
        let lib = libloading::Library::new(library_path).unwrap();
        let func: libloading::Symbol<NoBack> = lib.get(b"exec").unwrap();
        func()
    }
}

fn exec_add(library_path: &str, n1:i32, n2:i32) -> Result<i32, Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new(library_path).unwrap();
        let func: libloading::Symbol<AddFunc> = lib.get(b"add").unwrap();
        // Ok(func(isize::try_from(n1).unwrap(), isize::try_from(n2).unwrap()).try_into().unwrap())
        Ok(func(n1 as isize, n2 as isize) as i32)
    }
}


fn exec_cb(library_path: &str) -> Result<u32, Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new(library_path)?;
        let foo: libloading::Symbol<extern "C" fn(extern "C" fn() -> i32) -> u32> = lib.get(b"cb")?;
        Ok(foo(callback))
    }
}

fn main() {
    let library_path = env::args().nth(1).expect("USAGE: loading <LIB>");
    println!("Loading add() from {}", library_path);
    // let library_path = "./target/debug/wall.dll".to_string();
    exec_none(&library_path);
    let _rv1 = exec_add(&library_path, -1, 2);
    let _rv2 = exec_cb(&library_path);
    println!("Result: {} {}", _rv1.unwrap(), _rv2.unwrap())
}