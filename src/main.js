// System import
const { invoke } = window.__TAURI__.core
const { open } = window.__TAURI__.dialog;


// JS import
import { Application, Controller } from "/stimulus.min.js"


window.Stimulus = Application.start()

Stimulus.register("budget", class extends Controller {
    static targets = ['textInput', 'message', 'main']

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

    loadExpenses(e) {
        this.loadPart('_parts/_expenses.html', this.mainTarget)
    }

    loadUnits(e) {
        this.loadPart('_parts/_units.html', this.mainTarget)
    }

    loadPart(htmlPart, target) {
        fetch(htmlPart)
            .then(response => response.text())
            .then(html => {
                target.innerHTML = html
            })
    }

})