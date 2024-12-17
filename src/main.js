// System import
const { invoke } = window.__TAURI__.core
const { open } = window.__TAURI__.dialog;


// JS import
import { Application, Controller } from "/stimulus.min.js"


window.Stimulus = Application.start()

Stimulus.register("budget", class extends Controller {
    static targets = ['textInput', 'message']

    connect() {
    }

    async openFile(e) {
        const file = await open({
            multiple: false,
            directory: false,
        });

        if (file) {
            invoke("update_db_path", { path: file })
        }
    }

    async formSubmit(e) {
        e.preventDefault()
        this.messageTarget.innerHTML = await invoke("greet", { name: this.textInputTarget.value })
    }
})