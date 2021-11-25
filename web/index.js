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

function update(now) {
    app.tick(now);
    requestAnimationFrame(update);
}
requestAnimationFrame(update);

function resizeCanvasToDisplaySize(canvas, multiplier) {
    const width = window.innerWidth;
    const height = window.innerHeight;
    canvas.width = width;
    canvas.height = height;
    gl.viewport(0, 0, width, height);
}

resizeCanvasToDisplaySize(canvas, window.devicePixelRatio);
window.addEventListener("resize", function(e) {
    debounce(function() {
        resizeCanvasToDisplaySize(canvas, window.devicePixelRatio);
    }, 250);
});

var timerId;
function debounce(func, delay) {
    clearTimeout(timerId);
    timerId = setTimeout(() => func(), delay);
}