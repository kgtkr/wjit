const fs = require("fs/promises");

const wasmPath = "target/wasm32-unknown-unknown/debug/wjit.wasm";
const dumpWasm = process.argv.includes("--dump-wasm");

(async () => {
  const wasmBin = await fs.readFile(wasmPath);
  const wasmInstance = (await WebAssembly.instantiate(wasmBin, {})).instance;

  function stringToPtr(str) {
    const buf = Buffer.from(str);
    const ptr = wasmInstance.exports.alloc(buf.length + 1);
    const memoryView = new DataView(wasmInstance.exports.memory.buffer);

    for (let i = 0; i < buf.length; i++) {
      memoryView.setUint8(ptr + i, buf[i]);
    }
    memoryView.setUint8(ptr + buf.length, 0);
    return ptr;
  }

  function ptrToBuffer(ptr, len) {
    const buf = wasmInstance.exports.memory.buffer;
    return buf.slice(ptr, ptr + len);
  }

  const code = await fs.readFile(process.argv[2], { encoding: "utf8" });

  const irModulePtr = wasmInstance.exports.make_ir_module(stringToPtr(code));
  const compilerPtr = wasmInstance.exports.make_compiler(irModulePtr);

  const skeletonBinLenPtr = wasmInstance.exports.alloc(4);
  const skeletonBinPtr = wasmInstance.exports.compile_skeleton(
    compilerPtr,
    skeletonBinLenPtr
  );

  const memoryView = new DataView(wasmInstance.exports.memory.buffer);
  const skeletonBinLen = memoryView.getInt32(skeletonBinLenPtr, true);
  const skeletonBin = ptrToBuffer(skeletonBinPtr, skeletonBinLen);
  if (dumpWasm) {
    await fs.writeFile(`dump_wasm/skeleton.wasm`, Buffer.from(skeletonBin));
  }

  let table;
  const skeletonInstance = (
    await WebAssembly.instantiate(skeletonBin, {
      env: {
        compile_func: (idx) => {
          console.log("compile_func", idx);
          const funcBinLenPtr = wasmInstance.exports.alloc(4);
          const funcBinPtr = wasmInstance.exports.compile_func(
            compilerPtr,
            idx,
            funcBinLenPtr
          );
          const memoryView = new DataView(wasmInstance.exports.memory.buffer);
          const funcBinLen = memoryView.getInt32(funcBinLenPtr, true);
          const funcBin = ptrToBuffer(funcBinPtr, funcBinLen);
          if (dumpWasm) {
            require("fs").writeFileSync(
              `dump_wasm/${idx}.wasm`,
              Buffer.from(funcBin)
            );
          }
          const funcModule = new WebAssembly.Module(funcBin);
          new WebAssembly.Instance(funcModule, {
            env: {
              _table: table,
              println: (x) => {
                console.log(x);
                return 0;
              },
            },
          });
          return 0;
        },
      },
    })
  ).instance;
  table = skeletonInstance.exports._table;

  skeletonInstance.exports.main();
})();
