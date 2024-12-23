// System import
const { invoke } = window.__TAURI__.core
const { open, save } = window.__TAURI__.dialog;


// JS import
import { Application, Controller } from "/stimulus.min.js"

function renderTemplate(templateString, data) {
    return templateString.replace(/{{(.*?)}}/g, (match, p1) => {
        const key = p1.trim()
        return data[key] !== undefined ? data[key] : match
    });
}

async function fetchPart(htmlPart) {
    var result
    await fetch(htmlPart)
        .then(response => response.text())
        .then(html => {
            result = html
        })
    return result
}

async function generateFromFilePath(filePathString, data) {
    let strPrototype = await fetchPart(filePathString)
    return Array.isArray(data) ?
        data.map((obj) => renderTemplate(strPrototype, obj)).join() :
        renderTemplate(strPrototype, data)
}

function escapeHtmlAttribute(str) {
    return str.replace(/["'&<>]/g, function (char) {
        switch (char) {
            case '"':
                return '&quot;';
            case "'":
                return '&#39;';
            case '&':
                return '&amp;';
            case '<':
                return '&lt;';
            case '>':
                return '&gt;';
            default:
                return char;
        }
    });
}

window.Stimulus = Application.start()

Stimulus.register("budget", class extends Controller {
    static targets = ['textInput', 'message', 'main', 'sectionEditList', 'sectionEditAdd', 'sectionEditItem']

    connect() {
    }

    async openFile(e) {
        const file = await open({
            multiple: false,
            directory: false,
            filters: [{ name: "Longchamp Budget", extensions: ["lb"] }]
        })

        if (file) {
            invoke("update_db_path", { path: file })
        }
    }

    async createFile(e) {
        const file = await save({
            defaultPath: "budget.lb",
            filters: [{ name: "Longchamp Budget", extensions: ["lb"] }]
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

    async sectionEditListLoad() {
        let sectionList = JSON.parse(await invoke("section_list_without_group_load"))

        if (!sectionList) {
            return
        }

        sectionList = sectionList.map((section) => {
            section.title = escapeHtmlAttribute(section.title)
            section.color = escapeHtmlAttribute(section.color)
            return section
        })

        this.sectionEditListTarget.innerHTML = await generateFromFilePath('_parts/_components/_section-edit-item.html', sectionList)
    }

    loadExpenses(e) {
        this.loadPart('_parts/_windows/_expenses.html', this.mainTarget)
    }

    async loadSections(e) {
        this.loadPart('_parts/_windows/_sections.html', this.mainTarget)
        this.sectionEditListLoad()
    }

    async loadPart(htmlPart, target) {
        target.innerHTML = await fetchPart(htmlPart)
    }

})

Stimulus.register("section", class extends Controller {
    static targets = ['title', 'color']
    static values = {
        uid: String
    }

    sectionEdit(e) {
        if (!this.validate()) {
            return
        }
        console.log({ uid: this.uidValue, title: this.titleTarget.value, color: this.colorTarget.value })
    }

    validate() {
        return '' !== this.titleTarget.value.trim() && '' !== this.colorTarget.value.trim()
    }

})