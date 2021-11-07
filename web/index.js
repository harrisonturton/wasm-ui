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
    multiplier = multiplier || 1;
    const width = canvas.clientWidth * multiplier;
    const height = canvas.clientHeight * multiplier;
    if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
        gl.viewport(0, 0, width, height);
        return true;
    }
    return false;
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