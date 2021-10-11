import init, { generate_raw_data } from "./pkg/image_assemblifier.js";

const CANVAS = document.createElement("canvas");
const CANVAS_CTX = CANVAS.getContext("2d");

const FILE_INPUT = document.getElementById("file-input");
const PROCESS_BTN = document.getElementById("generate-btn");
// const PROGRESS_BAR = document.getElementById("progress-bar");
const COLS_INPUT = document.getElementById("num-cols-input");

const MODAL = {
    button: document.getElementById("modal-content-close"),
    content: document.getElementById("modal-content-text"),
    container: document.getElementById("modal-content-container"),
    root: document.getElementById("the-modal"),
    background: document.getElementById("modal-bg")
};

const NUM_REGEX = new RegExp("^[0-9]+$");

var _global = typeof window === 'object' && window.window === window ?
    window : typeof self === 'object' && self.self === self ?
    self : typeof global === 'object' && global.global === global ?
    global :
    this

let IMAGE_COLS = 100;

function setModal(visible) {
    if (visible) {
        MODAL.root.classList.add("is-active");
    } else {
        MODAL.root.classList.remove("is-active");
    }
}

function setModalContent(content, colorClass) {
    MODAL.container.classList.remove(["is-danger", "is-success", "is-warning", "is-primary", "is-link"]);
    MODAL.container.classList.add(colorClass);
    MODAL.content.innerHTML = content;
}

function main() {
    MODAL.button.addEventListener("click", () => { setModal(false) });
    MODAL.background.addEventListener("click", () => { setModal(false) });
    PROCESS_BTN.addEventListener("click", doProcess);
    FILE_INPUT.addEventListener("change", handleFileSelected, false);
}

function doProcess() {
    let desired_cols = COLS_INPUT.value;
    if (!NUM_REGEX.test(desired_cols)) {
        console.error("Invalid input");
        setModalContent("Invalid number of desired columns", "is-danger");
        setModal(true);
        return;
    }
    IMAGE_COLS = Number.parseInt(desired_cols);

    // PROGRESS_BAR.removeAttribute("value");
    let reader = new FileReader();
    reader.onload = (event) => {
        let img = new Image();
        img.onload = handleImageLoad;
        img.onerror = handleImageError;
        img.src = event.target.result;
    }
    reader.onerror = (event) => {
        console.error("The file could not be read", event);
        setModalContent("The file could not be read", "is-danger");
        setModal(true);
        // PROGRESS_BAR.setAttribute("value", "0");

    }
    reader.readAsDataURL(FILE_INPUT.files[0]);
}

function handleFileSelected(filePickedEvent) {
    if (filePickedEvent.target.files.length == 0) {
        document.getElementById("file-name").innerText = "";
        PROCESS_BTN.disabled = true;
    } else {
        document.getElementById("file-name").innerText = filePickedEvent.target.files[0].name;
        PROCESS_BTN.disabled = false;
    }
}

function handleImageLoad(event) {

    CANVAS.width = IMAGE_COLS;
    CANVAS.height = Math.round((event.target.height / event.target.width) * IMAGE_COLS * 0.43);

    CANVAS_CTX.drawImage(event.target, 0, 0, CANVAS.width, CANVAS.height);
    let data = CANVAS_CTX.getImageData(0, 0, CANVAS.width, CANVAS.height)

    console.log(CANVAS.width, CANVAS.height, data.data.length);
    let output = generate_raw_data(data.data, data.width, data.height);
    if (output.status == 0) {
        const blob = new Blob([output.message], { type: "text/plain;charset=utf-8" });
        _global.saveAs(blob, "generated.s");
    } else {
        console.error(output.message);
        setModalContent("An error occured during processing: " + output.message, "is-danger");
        setModal(true);
    }
    // PROGRESS_BAR.setAttribute("value", "0");
}

function handleImageError(event) {
    console.error("The image could not be loaded!", event);
    // PROGRESS_BAR.setAttribute("value", "0");
    setModalContent("The image could not be loaded", "is-danger");
    setModal(true);
}

document.addEventListener("DOMContentLoaded", () => {
    init().then(main);
});