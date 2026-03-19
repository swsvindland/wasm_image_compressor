import initWasm from "../wasm/pkg/lib/index";
import module_or_path from "wasm_image_compressor/lib/index_bg.wasm?url";

export default defineNuxtPlugin(async () => {
  await initWasm({ module_or_path });
});
