"use strict";
const fs = require("fs");

class Runner {
  constructor() {
    this.dumpWasm = process.argv.includes("--dump-wasm");

    const wasmPath = "target/wasm32-unknown-unknown/debug/wjit.wasm";
    const wasmBin = fs.readFileSync(wasmPath);
    const wasmModule = new WebAssembly.Module(wasmBin);
    this.wasmInstance = new WebAssembly.Instance(wasmModule);
  }

  stringToPtr(str) {
    const buf = Buffer.from(str);
    const ptr = this.wasmInstance.exports.alloc(buf.length + 1);
    const memoryView = new DataView(this.wasmInstance.exports.memory.buffer);

    for (let i = 0; i < buf.length; i++) {
      memoryView.setUint8(ptr + i, buf[i]);
    }
    memoryView.setUint8(ptr + buf.length, 0);
    return ptr;
  }

  ptrToBuffer(ptr, len) {
    const buf = this.wasmInstance.exports.memory.buffer;
    return buf.slice(ptr, ptr + len);
  }

  makeIrModule(code) {
    return this.wasmInstance.exports.make_ir_module(this.stringToPtr(code));
  }

  makeCompiler(irModule) {
    return this.wasmInstance.exports.make_compiler(irModule);
  }

  makeSkeletonModule(compiler) {
    const skeletonBinLenPtr = this.wasmInstance.exports.alloc(4);
    const skeletonBinPtr = this.wasmInstance.exports.compile_skeleton(
      compiler,
      skeletonBinLenPtr
    );
    const memoryView = new DataView(this.wasmInstance.exports.memory.buffer);
    const skeletonBinLen = memoryView.getInt32(skeletonBinLenPtr, true);
    const skeletonBin = this.ptrToBuffer(skeletonBinPtr, skeletonBinLen);
    if (this.dumpWasm) {
      fs.writeFileSync(`dump_wasm/skeleton.wasm`, Buffer.from(skeletonBin));
    }
    return new WebAssembly.Module(skeletonBin);
  }

  makeSkeltonInstance(compiler, skeletonModule) {
    const skeletonInstance = new WebAssembly.Instance(skeletonModule, {
      env: {
        compile_func: (idx) => {
          console.log("compile_func", idx);
          const funcBinLenPtr = this.wasmInstance.exports.alloc(4);
          const funcBinPtr = this.wasmInstance.exports.compile_func(
            compiler,
            idx,
            funcBinLenPtr
          );
          const memoryView = new DataView(
            this.wasmInstance.exports.memory.buffer
          );
          const funcBinLen = memoryView.getInt32(funcBinLenPtr, true);
          const funcBin = this.ptrToBuffer(funcBinPtr, funcBinLen);
          if (this.dumpWasm) {
            fs.writeFileSync(`dump_wasm/${idx}.wasm`, Buffer.from(funcBin));
          }
          const funcModule = new WebAssembly.Module(funcBin);
          new WebAssembly.Instance(funcModule, {
            env: {
              _table: skeletonInstance.exports._table,
              println: (x) => {
                console.log(x);
                return 0;
              },
            },
          });
          return 0;
        },
      },
    });
    return skeletonInstance;
  }
}

const code = fs.readFileSync(process.argv[2], { encoding: "utf8" });
const runner = new Runner();
const irModule = runner.makeIrModule(code);
const compiler = runner.makeCompiler(irModule);
const skeletonModule = runner.makeSkeletonModule(compiler);
const skeletonInstance = runner.makeSkeltonInstance(compiler, skeletonModule);

skeletonInstance.exports.main();
