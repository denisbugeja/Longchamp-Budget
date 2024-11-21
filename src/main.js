import { Application, Controller } from "/stimulus.min.js"
window.Stimulus = Application.start()
const { invoke } = window.__TAURI__.core

Stimulus.register("budget", class extends Controller {
    static targets = ['textInput', 'message']

    connect() {
    }

    async formSubmit(e) {
        e.preventDefault()
        this.messageTarget.innerHTML = await invoke("greet", { name: this.textInputTarget.value })
    }
})