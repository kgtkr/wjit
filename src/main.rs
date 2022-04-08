fn main() {}

use std::ffi::CString;
use std::os::raw::c_char;
use wjit::*;

#[no_mangle]
pub fn alloc(size: i32) -> *mut u8 {
    let mut buf = Vec::with_capacity(size as usize);
    buf.resize(size as usize, 0);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub fn make_compiler(code: *mut c_char) -> *mut compiler::Compiler {
    let code = unsafe { CString::from_raw(code) };
    let code = code.into_string().unwrap();
    let tokens = tokenizer::tokenize(code.as_str()).unwrap().1;
    let module = parser::parse(tokens.as_slice()).unwrap().1;
    let compiler = compiler::Compiler::new(module).unwrap();

    let result = Box::new(compiler);
    let result = Box::into_raw(result);

    std::mem::forget(result);

    result
}

#[no_mangle]
pub fn compile_skeleton(compiler: *mut compiler::Compiler, len: *mut i32) -> *const u8 {
    let compiler = unsafe { &*compiler };
    let module = compiler.compile_skeleton();
    let buf = parity_wasm::serialize(module).unwrap();
    let result = buf.as_ptr();
    unsafe {
        *len = buf.len() as i32;
    };

    std::mem::forget(buf);
    result
}

#[no_mangle]
pub fn compile_func(compiler: *mut compiler::Compiler, idx: i32, len: *mut i32) -> *const u8 {
    let compiler = unsafe { &*compiler };
    let module = compiler.compile_func_module(idx as usize);
    let buf = parity_wasm::serialize(module).unwrap();
    let result = buf.as_ptr();
    unsafe {
        *len = buf.len() as i32;
    };

    std::mem::forget(buf);
    result
}
