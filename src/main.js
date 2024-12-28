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
        data.map((obj) => renderTemplate(strPrototype, obj)).join('') :
        renderTemplate(strPrototype, data)
}

async function loadPart(htmlPart, target) {
    target.innerHTML = await fetchPart(htmlPart)
}

function escapeHtmlAttribute(str) {
    return str.toString().replace(/["'&<>]/g, function (char) {
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
    static targets = ['textInput', 'message', 'main']

    connect() {
    }

    async openFile(e) {
        const file = await open({
            multiple: false,
            directory: false,
            filters: [{ name: "Longchamp Budget", extensions: ["lb"] }]
        })

        if (file) {
            await invoke("update_db_path", { path: file })
        }
    }

    async createFile(e) {
        const file = await save({
            defaultPath: "budget.lb",
            filters: [{ name: "Longchamp Budget", extensions: ["lb"] }]
        })

        if (file) {
            await invoke("update_db_path", { path: file })
        }
    }

    async formSubmit(e) {
        e.preventDefault()
        this.messageTarget.innerHTML = await invoke("greet", { name: this.textInputTarget.value })
    }

    loadExpenses(e) {
        loadPart('_parts/_windows/_expenses.html', this.mainTarget)
    }

    loadSections(e) {
        loadPart('_parts/_windows/_sections.html', this.mainTarget)
    }
})


Stimulus.register("section", class extends Controller {
    static targets = ['title', 'color', 'sectionList']

    connect() {
    }

    sectionListTargetConnected(element) {
        this.sectionListLoad()
    }

    create(e) {
        e.preventDefault()
        if (!this.validateSection()) {
            return;
        }
        invoke("insert_new_section", { title: this.titleTarget.value, color: this.colorTarget.value })
        this.titleTarget.value = ''
        this.colorTarget.value = ''
        this.sectionListLoad()
    }

    async sectionListLoad() {
        let sectionList = JSON.parse(await invoke("section_list_load"))

        if (!sectionList) {
            return
        }

        sectionList = sectionList.map((section) => {
            section.title = escapeHtmlAttribute(section.title)
            section.color = escapeHtmlAttribute(section.color)
            return section
        })

        this.sectionListTarget.innerHTML = await generateFromFilePath('_parts/_components/_section-edit-item.html', sectionList)
    }

    validateSection() {
        return '' !== this.titleTarget.value.trim() && '' !== this.colorTarget.value.trim()
    }
})

Stimulus.register("section-edit", class extends Controller {
    static targets = ['title', 'color', 'delete']
    static outlets = ["section"]
    static values = {
        uid: String
    }

    connect() {
        this.deleteTarget.disabled = this.uidValue == 'group'
    }

    submit(e) {
        e.preventDefault()
    }

    update(e) {
        if (!this.validate()) {
            return
        }
        invoke("update_section", { uid: this.uidValue, title: this.titleTarget.value.trim(), color: this.colorTarget.value.trim() })
    }

    async delete(e) {
        if (!await invoke("is_allowed_to_delete_section", { uid: this.uidValue })) {
            alert("Tu ne peux pas supprimer cette section.\nElle est déja utilisée à une dépense.")
            return
        }
        invoke("delete_section", { uid: this.uidValue })
        this.sectionOutlet.sectionListLoad()
    }

    validate() {
        return '' !== this.titleTarget.value.trim() && '' !== this.colorTarget.value.trim()
    }
})

Stimulus.register("expense", class extends Controller {
    static targets = ['title', 'description', 'rate', 'unitPrice', 'expenseList']

    connect() {
    }

    expenseListTargetConnected(element) {
        this.expenseListLoad()
    }

    create(e) {
        e.preventDefault()
        if (!this.validate()) {
            alert('Les champs ne sont pas correctement remplis.')
            return;
        }

        invoke("insert_new_expense", { title: this.titleTarget.value, description: this.descriptionTarget.value, rate: this.rateTarget.value, unitPrice: this.unitPriceTarget.value })

        this.titleTarget.value = ''
        this.descriptionTarget.value = ''
        this.rateTarget.value = ''
        this.unitPriceTarget.value = ''

        this.expenseListLoad()
    }

    async expenseListLoad() {
        let expenseList = JSON.parse(await invoke("expense_list_load"))

        if (!expenseList) {
            return
        }

        expenseList = expenseList.map((expense) => {
            expense.title = escapeHtmlAttribute(expense.title)
            expense.description = escapeHtmlAttribute(expense.description)
            expense.rate = escapeHtmlAttribute(expense.rate)
            expense.unit_price = escapeHtmlAttribute(expense.unit_price)
            return expense
        })

        this.expenseListTarget.innerHTML = await generateFromFilePath('_parts/_components/_expense-edit-item.html', expenseList)
    }

    validate() {
        return '' !== this.titleTarget.value.trim()

            && '' !== this.rateTarget.value.trim()
            && parseFloat(this.rateTarget.value) > 0
            && parseFloat(this.rateTarget.value) <= 100

            && "" !== this.unitPriceTarget.value.trim()
            && parseFloat(this.unitPriceTarget.value) > 0
            && parseFloat(this.unitPriceTarget.value) <= 100
    }
})

Stimulus.register("expense-edit", class extends Controller {
    static targets = ['title', 'description', 'rate', 'unitPrice']
    static outlets = ["expense"]
    static values = {
        uid: String
    }

    submit(e) {
        e.preventDefault()
    }

    update(e) {
        if (!this.validate()) {
            return
        }
        invoke("update_expense", { uid: this.uidValue, title: this.titleTarget.value, description: this.descriptionTarget.value, rate: this.rateTarget.value, unitPrice: this.unitPriceTarget.value })
    }

    async delete(e) {
        if (!await invoke("is_allowed_to_delete_expense", { uid: this.uidValue })) {
            alert("Tu ne peux pas supprimer cette dépense.\nElle est déja utilisée à par une section.")
            return
        }
        invoke("delete_expense", { uid: this.uidValue })
        this.expenseOutlet.expenseListLoad()
    }

    validate() {
        return '' !== this.titleTarget.value.trim()

            && '' !== this.rateTarget.value.trim()
            && parseFloat(this.rateTarget.value) > 0
            && parseFloat(this.rateTarget.value) <= 100

            && "" !== this.unitPriceTarget.value.trim()
            && parseFloat(this.unitPriceTarget.value) > 0
            && parseFloat(this.unitPriceTarget.value) <= 100
    }
})