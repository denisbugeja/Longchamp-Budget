// System import
const { invoke } = window.__TAURI__.core
const { open, save } = window.__TAURI__.dialog;


// JS import
import { Application, Controller } from "/stimulus.min.js"


window.Stimulus = Application.start()

Stimulus.register("budget", class extends Controller {
    static targets = ['textInput', 'message', 'main', 'sectionEditList', 'sectionEditAdd', 'sectionEditItem']

    connect() {
    }

    async openFile(e) {
        const file = await open({
            multiple: false,
            directory: false,
        })

        if (file) {
            invoke("update_db_path", { path: file })
        }
    }

    async createFile(e) {
        const file = await save({
            defaultPath: "budget.db",
            filters: [{ name: "Longchamp Budget", extensions: ["db"] }]
        })

        if (file) {
            invoke("update_db_path", { path: file })
        }
    }

    async formSubmit(e) {
        e.preventDefault()
        this.messageTarget.innerHTML = await invoke("greet", { name: this.textInputTarget.value })
    }

    sectionEditAddSubmit(e) {
        e.preventDefault()
        alert('coucou')
    }

    sectionEditListLoad() {
        const sectionList = invoke("section_list_load")
    }

    loadExpenses(e) {
        this.loadPart('_parts/_windows/_expenses.html', this.mainTarget)
    }

    async loadSections(e) {
        this.loadPart('_parts/_windows/_sections.html', this.mainTarget)
        const sectionList = await invoke("section_list_load")
        console.log(sectionList)
    }

    loadPart(htmlPart, target) {
        fetch(htmlPart)
            .then(response => response.text())
            .then(html => {
                target.innerHTML = html
            })
    }

})