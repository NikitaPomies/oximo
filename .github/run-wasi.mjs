import { readFile } from "node:fs/promises";
import { WASI } from "node:wasi";

const [, , wasmPath, ...args] = process.argv;
if (wasmPath === undefined) {
  throw new Error("usage: run-wasi.mjs <module.wasm> [args...]");
}

const wasi = new WASI({
  version: "preview1",
  args: [wasmPath, ...args],
  env: process.env,
});
const module = await WebAssembly.compile(await readFile(wasmPath));
const instance = await WebAssembly.instantiate(module, {
  wasi_snapshot_preview1: wasi.wasiImport,
});
wasi.start(instance);
