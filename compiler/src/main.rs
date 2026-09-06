use std::ffi::OsString;

pub fn main() {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    std::process::exit(meowy::driver::run(args));
}
