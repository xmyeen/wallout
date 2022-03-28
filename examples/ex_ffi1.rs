use std::{env, os::raw::c_char, ffi::CString};
use libloading;

/**
 *  如果使用c的数结构，参考此页。
 *  https://static.rust-lang.org/doc/master/book/ffi.html
 * 
 */

/*
extern "C" fn callback() -> i32 {
    30i32
}

type NoBack = fn();
type AddFunc = fn(isize, isize) -> isize;


*/

// enum ApiDefs {
//     StartServer(b"start_server"),
// }

enum WalloutApi {
    StartServer,
}

struct WalloutFFI {
    lib: libloading::Library,
}

impl WalloutFFI {
    pub unsafe fn new(lib_path: &str) -> Self {
        Self {
            lib: libloading::Library::new(lib_path).unwrap()
        }
    }

    fn name(&self, api_en: WalloutApi) -> &[u8] {
        match api_en {
            WalloutApi::StartServer => b"start_server",
        }
    }

    pub unsafe fn start_server(&self) {
        let name = self.name(WalloutApi::StartServer);
        let start_server_fn: libloading::Symbol<fn(*const c_char)> = self.lib.get(name).unwrap();

        let appcfg_c_str = CString::new("config/app").unwrap();
        start_server_fn(appcfg_c_str.as_ptr() as *const c_char);
    }
}


fn main() {
    let library_path = env::args().nth(1).expect("USAGE: loading <LIB>");
    unsafe {
        WalloutFFI::new(library_path.as_str()).start_server();
    }
}