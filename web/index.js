import * as wasm from "core";

const canvas = document.getElementById("app");
if (!canvas) {
    console.error("failed to get canvas element");
}

const gl = canvas.getContext("webgl");
if (!gl) {
    console.error("failed to get webgl context")
}

let app = wasm.start("app");