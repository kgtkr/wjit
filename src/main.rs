use std::ffi::CString;
use std::os::raw::c_char;
use wjit::*;

fn main() {}

#[no_mangle]
pub fn alloc(size: i32) -> *mut u8 {
    let mut buf = Vec::with_capacity(size as usize);
    buf.resize(size as usize, 0);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub unsafe fn make_ir_module(code: *mut c_char) -> *mut ir::Module {
    let code = CString::from_raw(code);
    let code = code.into_string().unwrap();
    let tokens = tokenizer::tokenize(code.as_str()).unwrap().1;
    let module = parser::parse(tokens.as_slice()).unwrap().1;
    let module = ir_generator::generate(&module);

    let result = Box::new(module);
    let result = Box::into_raw(result);

    std::mem::forget(result);

    result
}

#[no_mangle]
pub unsafe fn make_compiler<'a>(module: *mut ir::Module) -> *mut compiler::Compiler<'a> {
    let module = &*module;
    let compiler = compiler::Compiler::new(module);

    let result = Box::new(compiler);
    let result = Box::into_raw(result);

    std::mem::forget(result);

    result
}

#[no_mangle]
pub unsafe fn compile_skeleton(compiler: *mut compiler::Compiler, len: *mut i32) -> *const u8 {
    let compiler = &*compiler;
    let module = compiler.compile_skeleton();
    let buf = parity_wasm::serialize(module).unwrap();
    let result = buf.as_ptr();
    *len = buf.len() as i32;

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

#[no_mangle]
pub fn make_interpreter(
    module: &ir::Module,
) -> *mut interpreter::Interpreter<interpreter::WasmBuiltin> {
    let interpreter = interpreter::Interpreter::new(module, interpreter::WasmBuiltin);
    let interpreter = Box::new(interpreter);
    let interpreter = Box::into_raw(interpreter);

    std::mem::forget(interpreter);
    interpreter
}

#[no_mangle]
pub unsafe fn interpreter_call_func(
    interpreter: &mut interpreter::Interpreter<interpreter::WasmBuiltin>,
    func: usize,
    args_count: usize,
    args: *const i32,
) -> i32 {
    let args = if args == std::ptr::null() {
        &[]
    } else {
        std::slice::from_raw_parts(args, args_count)
    };
    interpreter.call(func, args)
}
