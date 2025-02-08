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

    loadMatrix(e) {
        loadPart('_parts/_windows/_matrix.html', this.mainTarget)
    }
})


Stimulus.register("section", class extends Controller {
    static targets = ['title', 'color', 'sectionList']

    connect() {
    }

    sectionListTargetConnected(element) {
        this.sectionListLoad()
    }

    usedSectionExpense = null

    async getUsedSectionExpense() {
        if (null === this.usedSectionExpense) {
            this.usedSectionExpense = JSON.parse(await invoke("get_section_expense_from_expenses_instances"))
        }
        return this.usedSectionExpense
    }

    async create(e) {
        e.preventDefault()
        if (!this.validateSection()) {
            return;
        }
        await invoke("insert_new_section", { title: this.titleTarget.value, color: this.colorTarget.value })
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

    used = null

    async connect() {
        this.deleteTarget.disabled = this.uidValue == 'group' || await this.isUsed()
    }

    async isUsed() {
        if (null === this.used) {
            const usedSectionExpense = await this.sectionOutlet.getUsedSectionExpense(),
                sectionUid = this.uidValue,
                usedSectionList = (usedSectionExpense).filter((sectionExpense) => sectionExpense.uid_section == sectionUid)

            this.used = 0 !== usedSectionList.length
        }
        return this.used
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
        if (await this.isUsed()) {
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
    static targets = ['title', 'description', 'rate', 'unitPrice', 'expenseList', 'sectionList', 'section']

    usedSectionExpense = null
    associatedSectionExpense = null
    sectionList = null

    async getUsedSectionExpense() {
        if (null === this.usedSectionExpense) {
            this.usedSectionExpense = JSON.parse(await invoke("get_section_expense_from_expenses_instances"))
        }
        return this.usedSectionExpense
    }

    async getAssociatedSectionExpense() {
        if (null === this.associatedSectionExpense) {
            this.associatedSectionExpense = JSON.parse(await invoke("get_section_expense"))
        }
        return this.associatedSectionExpense
    }

    async getSectionList() {
        if (null === this.sectionList) {
            this.sectionList = JSON.parse(await invoke("section_list_load"))
        }
        return this.sectionList
    }

    expenseListTargetConnected(element) {
        this.expenseListLoad()
    }

    async create(e) {
        e.preventDefault()

        if (!this.validate()) {
            if (!this.hasAtLeastOneSectionChecked()) {
                this.sectionListTarget.classList.add('invalid')
            }
            return
        }

        this.sectionListTarget.classList.remove('invalid')

        const sectioncheckboxList = JSON.stringify(Array.from(
            this.sectionTargets
                .filter((section) => section.checked)
                .map((section) => section.value)
        ))


        await invoke("insert_new_expense", { title: this.titleTarget.value, description: this.descriptionTarget.value, rate: this.rateTarget.value, unitPrice: this.unitPriceTarget.value, sectionList: sectioncheckboxList })

        // force reload or relationship from database
        this.associatedSectionExpense = null
        this.usedSectionExpense = null

        this.titleTarget.value = ''
        this.descriptionTarget.value = ''
        this.rateTarget.value = ''
        this.unitPriceTarget.value = ''
        this.sectionTargets.forEach((section) => section.checked = false)

        this.expenseListLoad()
    }

    async sectionListTargetConnected(element) {
        const sectionList = await this.getSectionList()
        element.innerHTML = await generateFromFilePath('_parts/_components/_expense-create-item-sections.html', sectionList)
    }

    async expenseListLoad() {

        let expenseList = JSON.parse(await invoke("expense_list_load"))

        if (!expenseList) {
            return
        }

        let sectionList = (await this.getSectionList()).map((section) => {
            section.title = escapeHtmlAttribute(section.title)
            return section
        })

        const sectionCheckboxListHtml = await generateFromFilePath('_parts/_components/_expense-edit-item-sections.html', sectionList)

        expenseList = expenseList.map((expense) => {
            expense.title = escapeHtmlAttribute(expense.title)
            expense.description = escapeHtmlAttribute(expense.description)
            expense.rate = escapeHtmlAttribute(expense.rate)
            expense.unit_price = escapeHtmlAttribute(expense.unit_price)
            expense.section_list_html = sectionCheckboxListHtml
            return expense
        })

        this.expenseListTarget.innerHTML = await generateFromFilePath('_parts/_components/_expense-edit-item.html', expenseList)
    }

    hasAtLeastOneSectionChecked() {
        return 0 != this.sectionTargets.filter((section) => section.checked).length
    }

    isRateTargetValid() {
        return '' !== this.rateTarget.value.trim()
            && !isNaN(this.rateTarget.value)
            && parseFloat(this.rateTarget.value) >= 0
            && parseFloat(this.rateTarget.value) <= 100
    }

    isTitleTargetValid() {
        return '' !== this.titleTarget.value.trim()
    }

    isUnitPriceTargetValid() {
        return "" !== this.unitPriceTarget.value.trim()
            && !isNaN(this.unitPriceTarget.value)
            && parseFloat(this.unitPriceTarget.value) >= 0
    }

    validate() {
        return this.isTitleTargetValid()
            && this.isRateTargetValid()
            && this.isUnitPriceTargetValid()
            && this.hasAtLeastOneSectionChecked()
    }
})

Stimulus.register("expense-edit", class extends Controller {
    static targets = ['title', 'description', 'rate', 'unitPrice', 'sectionList', 'section', 'delete']
    static outlets = ["expense"]
    static values = {
        uid: String
    }

    used = null

    async sectionTargetConnected(element) {
        const sectionUid = element.value,
            expenseUid = this.uidValue,
            associatedSectionExpense = await this.expenseOutlet.getAssociatedSectionExpense(),
            used = await this.isUsed()

        element.disabled = used
        element.checked = used || 0 != associatedSectionExpense
            .filter((sectionExpense) => sectionExpense.uid_expense == expenseUid && sectionExpense.uid_section == sectionUid)
            .length
    }

    async isUsed() {
        if (null === this.used) {
            const usedSectionExpense = await this.expenseOutlet.getUsedSectionExpense(),
                expenseUid = this.uidValue,
                usedSectionList = (usedSectionExpense).filter((sectionExpense) => sectionExpense.uid_expense == expenseUid)

            this.used = 0 !== usedSectionList.length
        }
        return this.used
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

    async deleteTargetConnected(element) {
        element.disabled = await this.isUsed()
    }

    updateAssociation(e) {
        if (!this.hasAtLeastOneSectionChecked()) {
            this.sectionListTarget.classList.add('invalid')
            return
        }

        this.sectionListTarget.classList.remove('invalid')

        const sectioncheckboxList = JSON.stringify(Array.from(
            this.sectionTargets
                .filter((section) => section.checked)
                .map((section) => section.value)
        ))

        invoke("update_expense_section_association", { uid: this.uidValue, sectionList: sectioncheckboxList })
    }

    async delete(e) {
        if (await this.isUsed()) {
            alert("Tu ne peux pas supprimer cette dépense.\nElle est déja utilisée à par une section.")
            return
        }
        invoke("delete_expense", { uid: this.uidValue })
        this.expenseOutlet.expenseListLoad()
    }

    isRateTargetValid() {
        return '' !== this.rateTarget.value.trim()
            && !isNaN(this.rateTarget.value)
            && parseFloat(this.rateTarget.value) >= 0
            && parseFloat(this.rateTarget.value) <= 100
    }

    isTitleTargetValid() {
        return '' !== this.titleTarget.value.trim()
    }

    isUnitPriceTargetValid() {
        return "" !== this.unitPriceTarget.value.trim()
            && !isNaN(this.unitPriceTarget.value)
            && parseFloat(this.unitPriceTarget.value) >= 0
    }

    hasAtLeastOneSectionChecked() {
        return 0 != this.sectionTargets.filter((section) => section.checked).length
    }

    validate() {
        return this.isTitleTargetValid()
            && this.isRateTargetValid()
            && this.isUnitPriceTargetValid()
    }
})

Stimulus.register("matrix", class extends Controller {
    static targets = ['sectionList']
    static outlets = ["matrix-section"]

    sectionList = null

    async getSectionList() {
        if (null === this.sectionList) {
            this.sectionList = JSON.parse(await invoke("section_list_load"))
        }
        return this.sectionList
    }

    connect() {
    }

    sectionListTargetConnected(element) {
        this.sectionListLoad()
    }

    async sectionListLoad() {
        let sectionList = await this.getSectionList()

        if (!sectionList) {
            return
        }

        sectionList = sectionList.map((section) => {
            section.title = escapeHtmlAttribute(section.title)
            section.color = escapeHtmlAttribute(section.color)
            return section
        })

        this.sectionListTarget.innerHTML = await generateFromFilePath('_parts/_components/_matrix_section.html', sectionList)
    }

    async refreshAllData() {
        this.sectionList = null
        await this.getSectionList()
        this.matrixSectionOutlets.forEach((outlet) => { outlet.sectionRefresh() })
    }
})

Stimulus.register("matrix-section", class extends Controller {
    static targets = ['expenseList', 'expenseInstanceList', 'expenseGroupInstanceList', 'sectionMemberCount']
    static outlets = ["matrix"]
    static values = {
        uid: String
    }

    expenseList = null
    usedExpenseList = null
    usedGroupExpenseList = null

    async getExpenseList() {
        if (null === this.expenseList) {
            this.expenseList = JSON.parse(await invoke("get_section_expense_from_expenses_instances_section", { sectionUid: this.uidValue }))
        }
        return this.expenseList
    }

    async getUsedExpenseList() {
        if (null === this.usedExpenseList) {
            this.usedExpenseList = JSON.parse(await invoke("get_calculated_expenses", { sectionUid: this.uidValue }))
        }
        return this.usedExpenseList
    }

    async getGroupUsedExpenseList() {
        if (null === this.usedGroupExpenseList) {
            this.usedGroupExpenseList = JSON.parse(await invoke("get_group_calculated_expenses", {}))
        }
        return this.usedGroupExpenseList
    }

    async getMembersCount() {
        return await invoke("get_members_count", { sectionUid: this.uidValue })
    }

    async expenseListTargetConnected() {
        await this.expenseListLoad()
    }

    async sectionMemberCountTargetConnected() {
        await this.loadSectionMembersCount()
    }

    async expenseInstanceListTargetConnected() {
        await this.expenseInstanceListLoad()
    }

    async expenseGroupInstanceListTargetConnected() {
        await this.expenseGroupInstanceListLoad()
    }

    async expenseListLoad() {
        let expenseList = await this.getExpenseList()
        this.expenseListTarget.innerHTML = await generateFromFilePath('_parts/_components/_matrix_section_expense.html', expenseList)
    }

    async loadSectionMembersCount() {
        this.sectionMemberCountTarget.value = await this.getMembersCount()
    }

    async expenseInstanceListLoad() {
        let expenseInstanceList = await this.getUsedExpenseList()
        this.expenseInstanceListTarget.innerHTML = await generateFromFilePath('_parts/_components/_matrix_section_expense_instance.html', expenseInstanceList)
    }

    async expenseGroupInstanceListLoad() {
        if ('group' != this.uidValue) {
            return
        }

        let groupExpenseInstanceList = await this.getGroupUsedExpenseList()
        this.expenseGroupInstanceListTarget.innerHTML = await generateFromFilePath('_parts/_components/_matrix_section_group_expense_instance.html', groupExpenseInstanceList)
    }

    async addExpenseInstance(e) {
        await invoke("add_expense_instance", { sectionUid: this.uidValue, expenseId: e.target.getAttribute('data-expense-id') })
        this.triggerGlobalRefresh()
    }

    async updateMemberCount(e) {
        let targetValue = e.target.value
        if (!this.validateMemberCount(targetValue)) {
            return;
        }
        await invoke("update_members_count", { uid: this.uidValue, membersCount: targetValue })
        this.triggerGlobalRefresh()
    }

    validateMemberCount(targetValue) {
        return !isNaN(targetValue) && targetValue >= 0
    }

    async triggerGlobalRefresh() {
        this.expenseList = null
        this.usedExpenseList = null
        this.usedGroupExpenseList = null
        await this.matrixOutlet.refreshAllData()
    }

    sectionRefresh() {
        this.loadSectionMembersCount()
        this.expenseListLoad()
        this.expenseInstanceListLoad()
        this.expenseGroupInstanceListLoad()
    }
})

Stimulus.register("matrix-expense-instance", class extends Controller {
    static targets = ["unitPrice", "units", "rate"]
    static outlets = ["matrix-section"]
    static values = {
        uid: String
    }

    async updateExpenseInstance() {
        if (!this.validate()) {
            return;
        }
        await invoke("update_expense_instance", {
            uidExpenseInstance: this.uidValue,
            unitPrice: this.unitPriceTarget.value,
            units: this.unitsTarget.value,
            rate: this.rateTarget.value,
        })
        await this.matrixSectionOutlet.triggerGlobalRefresh()
    }

    unitPriceValid() {
        return '' === this.unitPriceTarget.value.trim()
            || (
                !isNaN(this.unitPriceTarget.value)
                && parseFloat(this.unitPriceTarget.value) > 0
            )
    }

    unitsValid() {
        return '' === this.unitsTarget.value.trim()
            || !isNaN(this.unitsTarget.value)
    }

    rateValid() {
        return '' === this.rateTarget.value.trim()
            || (
                !isNaN(this.rateTarget.value)
                && parseInt(this.rateTarget.value) >= 0
                && parseInt(this.rateTarget.value) <= 100
            )
    }

    validate() {
        return this.unitPriceValid()
            && this.unitsValid()
            && this.rateValid()
    }
})