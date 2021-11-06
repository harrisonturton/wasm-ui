import * as wasm from "core";

const canvas = document.getElementById("app");
if (!canvas) {
    console.error("failed to get canvas element");
}

const gl = canvas.getContext("webgl");
if (!gl) {
    console.error("failed to get webgl context")
}

let app = wasm.create("app");

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

document.addEventListener("mousemove", function(e) {
    app.on_mouse_move(e.clientX, e.clientY);
})

document.addEventListener("keydown", function(e) {
    console.log(`${e.key} is down!`);
    switch (e.key) {
        case ' ':
            app.on_key_down(wasm.Key.Spacebar);
            break;
        case 'ArrowUp':
            app.on_key_down(wasm.Key.ArrowUp);
            break;
        case 'ArrowDown':
            app.on_key_down(wasm.Key.ArrowDown);
            break;
        case 'ArrowLeft':
            app.on_key_down(wasm.Key.ArrowLeft);
            break;
        case 'ArrowRight':
            app.on_key_down(wasm.Key.ArrowRight);
            break;
        case 'w':
            app.on_key_down(wasm.Key.W);
            break;
        case 'a':
            app.on_key_down(wasm.Key.A);
            break;
        case 's':
            app.on_key_down(wasm.Key.S);
            break;
        case 'd':
            app.on_key_down(wasm.Key.D);
            break;
    }
})

document.addEventListener("keyup", function(e) {
    console.log(`${e.key} is up!`);
    switch (e.key) {
        case ' ':
            app.on_key_up(wasm.Key.Spacebar);
            break;
        case 'ArrowUp':
            app.on_key_up(wasm.Key.ArrowUp);
            break;
        case 'ArrowDown':
            app.on_key_up(wasm.Key.ArrowDown);
            break;
        case 'ArrowLeft':
            app.on_key_up(wasm.Key.ArrowLeft);
            break;
        case 'ArrowRight':
            app.on_key_up(wasm.Key.ArrowRight);
            break;
        case 'w':
            app.on_key_up(wasm.Key.W);
            break;
        case 'a':
            app.on_key_up(wasm.Key.A);
            break;
        case 's':
            app.on_key_up(wasm.Key.S);
            break;
        case 'd':
            app.on_key_up(wasm.Key.D);
            break;
    }
})

var timerId;
function debounce(func, delay) {
    clearTimeout(timerId);
    timerId = setTimeout(() => func(), delay);
}