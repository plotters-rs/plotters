import { Chart, drawPower } from "wasm-demo";

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");

// Setup canvas to properly handle high DPI.
canvas.style.width = canvas.width + "px";
canvas.style.height = canvas.height + "px";
const dpr = window.devicePixelRatio || 1;
const rect = canvas.getBoundingClientRect();
canvas.width *= dpr;
canvas.height *= dpr;
ctx.scale(dpr, dpr);

const coord = document.getElementById("coord");

let chart = undefined;
window.addEventListener("mousemove", (event) => {
	if (chart === undefined) {
        return;
    }

    const point = chart.coord(event.offsetX, event.offsetY);

    coord.innerText = (point)
        ? `(${point.x.toFixed(3)}, ${point.y.toFixed(3)})`
        : "Mouse pointer is out of range";
});

const select = document.getElementById("pow");
const status = document.getElementById("status");

const updatePlot = () => {
    const selected = select.selectedOptions[0];
	status.innerText = `Rendering ${selected.innerText}...`;
	setTimeout(() => {
		chart = undefined;
		const start = performance.now();
        chart = (selected.value == "mandelbrot")
            ? Chart.mandelbrot(canvas)
            : Chart.power("canvas", Number(selected.value));
		const end = performance.now();
		status.innerText = `Rendered ${selected.innerText} in ${end - start}ms`;
	}, 5);
};

status.innerText = "WebAssembly loaded!";

updatePlot();

select.addEventListener("change", updatePlot);
