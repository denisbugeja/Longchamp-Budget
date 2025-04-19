// System import
const { invoke } = window.__TAURI__.core
const { open, save } = window.__TAURI__.dialog
const GROUP_ID = 'group'

// JS import
import { Application, Controller } from "/stimulus.min.js"

let assetPath = {}

function renderTemplate(templateString, data, raw = false) {
    return templateString.replace(/{{(.*?)}}/g, (match, p1) => {
        const key = p1.trim()
        return raw ? data[key] ?? '' : escapeHtmlAttribute(data[key] ?? '')
    })
}

function getSelector(element) {
    if (!(element instanceof Element)) return

    const path = []
    while (element && Node.ELEMENT_NODE === element.nodeType) {
        let selector = element.nodeName.toLowerCase()
        if (element.id) {
            selector += `#${element.id}`
            path.unshift(selector)

            return path.join(" > ")
        } else {
            let sibling = element,
                nth = 1
            while (sibling.previousElementSibling) {
                sibling = sibling.previousElementSibling
                if (sibling.nodeName.toLowerCase() === selector) nth++
            }
            if (nth !== 1) selector += `:nth-of-type(${nth})`
        }
        path.unshift(selector)
        element = element.parentNode
    }
    return path.join(" > ")
}

function renderElement(element, content) {
    let focusedElement = document.activeElement,
        focusedElementString = (focusedElement) ? getSelector(focusedElement) : '',
        targetElement = null
    element.innerHTML = content
    if ('' !== focusedElementString) {
        targetElement = document.querySelector(focusedElementString)
        if (null !== targetElement) {
            targetElement.focus()
        }
    }
}

async function fetchPart(htmlPart) {

    if (!assetPath[htmlPart]) {
        assetPath[htmlPart] = await invoke('read_asset', { path: htmlPart })
    }
    return assetPath[htmlPart]
}

async function generateFromFilePath(filePathString, data, raw = false) {
    let strPrototype = await fetchPart(filePathString)
    return Array.isArray(data) ?
        data.map((obj) => renderTemplate(strPrototype, obj, raw)).join('') :
        renderTemplate(strPrototype, data, raw)
}

async function loadPart(htmlPart, target) {
    renderElement(target, await fetchPart(htmlPart))
}

function escapeHtmlAttribute(str) {
    return str.toString().replace(/["'&<>]/g, (char) => ({ '"': '&quot;', "'": '&#39;', '&': '&amp;', '<': '&lt;', '>': '&gt;' }[char] ?? char))
}

window.Stimulus = Application.start()

Stimulus.register("budget", class extends Controller {
    static targets = ['textInput', 'message', 'main', 'links']

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
            this.linksTarget.classList.remove('d-none')
        }
    }

    async createFile(e) {
        const file = await save({
            defaultPath: "budget.lb",
            filters: [{ name: "Longchamp Budget", extensions: ["lb"] }]
        })

        if (file) {
            await invoke("update_db_path", { path: file })
            this.linksTarget.classList.remove('d-none')
        }
    }

    loadExpenses() {
        loadPart('_parts/_windows/_expenses.html', this.mainTarget)
    }

    loadSections() {
        loadPart('_parts/_windows/_sections.html', this.mainTarget)
    }

    loadMatrix() {
        loadPart('_parts/_windows/_matrix.html', this.mainTarget)
    }
})


Stimulus.register("section", class extends Controller {
    static targets = ['title', 'color', 'sectionList', 'sectionMembersCount', 'sectionAdultsCount']
    static outlets = ["budget"]

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
        if (!this.validate()) {
            return
        }
        await invoke("insert_new_section", { title: this.titleTarget.value, color: this.colorTarget.value, membersCount: parseInt(this.sectionMembersCountTarget.value), adultsCount: parseInt(this.sectionAdultsCountTarget.value) })
        this.budgetOutlet.loadSections()
    }

    async sectionListLoad() {
        let sectionList = JSON.parse(await invoke("section_list_load"))

        if (!sectionList) {
            return
        }

        renderElement(this.sectionListTarget, await generateFromFilePath('_parts/_components/_section-edit-item.html', sectionList))
    }

    validateTitle() {
        return '' !== this.titleTarget.value.trim()
    }

    validateColor() {
        return '' !== this.colorTarget.value.trim()
    }

    validateMembers() {
        return '' !== this.sectionMembersCountTarget.value.trim()
            && !isNaN(this.sectionMembersCountTarget.value)
    }

    validateAdults() {
        return '' !== this.sectionAdultsCountTarget.value.trim()
            && !isNaN(this.sectionAdultsCountTarget.value)
    }

    validate() {
        return this.validateTitle()
            && this.validateColor()
            && this.validateMembers()
            && this.validateAdults()
    }
})

Stimulus.register("section-edit", class extends Controller {
    static targets = ['title', 'color', 'delete', 'sectionMembersCount', 'sectionAdultsCount']
    static outlets = ["section"]
    static values = {
        uid: String
    }

    async isUsed() {
        let expenseList = JSON.parse(await invoke("get_section_expense_from_expenses_instances_and_section", { sectionUid: this.uidValue })) ?? []
        return 0 !== expenseList.length
    }

    async deleteTargetConnected() {
        this.deleteTarget.disabled = this.uidValue == GROUP_ID || await this.isUsed()
    }

    sectionMembersCountTargetConnected() {
        if (GROUP_ID === this.uidValue) {
            this.sectionMembersCountTarget.setAttribute('readonly', 'readonly')
        }
    }

    submit(e) {
        e.preventDefault()
    }

    update(e) {
        if (!this.validate()) {
            return
        }
        invoke("update_section", { uid: this.uidValue, title: this.titleTarget.value.trim(), color: this.colorTarget.value.trim(), membersCount: parseInt(this.sectionMembersCountTarget.value), adultsCount: parseInt(this.sectionAdultsCountTarget.value) })
        this.sectionOutlet.sectionListLoad()
    }

    async delete(e) {
        if (await this.isUsed()) {
            alert("Tu ne peux pas supprimer cette section.\nElle est déja utilisée à une dépense.")
            return
        }
        invoke("delete_section", { uid: this.uidValue })
        this.sectionOutlet.sectionListLoad()
    }

    validateTitle() {
        return '' !== this.titleTarget.value.trim()
    }

    validateColor() {
        return '' !== this.colorTarget.value.trim()
    }

    validateMembers() {
        return '' !== this.sectionMembersCountTarget.value.trim()
            && !isNaN(this.sectionMembersCountTarget.value)
    }

    validateAdults() {
        return '' !== this.sectionAdultsCountTarget.value.trim()
            && !isNaN(this.sectionAdultsCountTarget.value)
    }

    validate() {
        return this.validateTitle()
            && this.validateColor()
            && this.validateMembers()
            && this.validateAdults()
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
        this.rateTarget.value = 100
        this.unitPriceTarget.value = ''
        this.sectionTargets.forEach((section) => section.checked = false)

        this.expenseListLoad()
    }

    async sectionListTargetConnected(element) {
        const sectionList = await this.getSectionList()
        renderElement(element, await generateFromFilePath('_parts/_components/_expense-create-item-sections.html', sectionList))
    }

    async expenseListLoad() {

        let expenseList = JSON.parse(await invoke("expense_list_load"))

        if (!expenseList) {
            return
        }

        renderElement(this.expenseListTarget, await generateFromFilePath('_parts/_components/_expense-edit-item.html', expenseList))
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

    async isUsed() {
        let expenseFromInstance = JSON.parse(await invoke("get_section_expense_from_instances_by_expense", { expenseUid: this.uidValue }))
        return 0 !== expenseFromInstance.length
    }

    async sectionListTargetConnected() {
        let sectionList = JSON.parse(await invoke("section_list_load"))
        renderElement(this.sectionListTarget, await generateFromFilePath('_parts/_components/_expense-edit-item-sections.html', sectionList))
    }

    async sectionTargetConnected(section) {
        let expenseFromInstance = JSON.parse(await invoke("get_section_expense_from_instance", { sectionUid: section.value, expenseUid: this.uidValue })),
            expenseFromAssociation = JSON.parse(await invoke("get_section_expense_from_association", { sectionUid: section.value, expenseUid: this.uidValue }))

        section.disabled = 0 !== expenseFromInstance.length
        section.checked = 0 !== expenseFromAssociation.length
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
            alert("Tu ne peux pas supprimer cette dépense.\nElle est déja utilisée par une section.")
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

    async getSectionList() {
        return JSON.parse(await invoke("section_list_load"))
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

        renderElement(this.sectionListTarget, await generateFromFilePath('_parts/_components/_matrix_section.html', sectionList))
    }

    async refreshAllData() {
        this.matrixSectionOutlets.forEach((outlet) => { outlet.sectionRefresh() })
    }
})

Stimulus.register("matrix-section", class extends Controller {
    static targets = ['expenseList', 'expenseInstanceList', 'expenseGroupRatioTotal', 'expenseGroupInstanceList', 'expenseGroupInstanceListContainer', 'sectionMembersCount', 'sectionAdultsCount', 'expenseInstanceGroupTotal', 'expenseInstanceTotal', 'expenseInstanceMemberTotal']
    static outlets = ['matrix']
    static values = {
        uid: String
    }

    async getExpenseList() {
        return JSON.parse(await invoke("get_section_expense_from_expenses_instances_section", { sectionUid: this.uidValue }))
    }

    async getUsedExpenseList() {
        return JSON.parse(await invoke("get_calculated_expenses", { sectionUid: this.uidValue }))
    }

    async getGroupUsedExpenseList() {
        return JSON.parse(await invoke("get_group_calculated_expenses", {}))
    }

    async getMembersCount() {
        return await invoke("get_members_count", { sectionUid: this.uidValue })
    }

    async getAdultsCount() {
        return await invoke("get_adults_count", { sectionUid: this.uidValue })
    }

    async getTotal() {
        return JSON.parse(await invoke("get_sum_calculated_expenses", { sectionUid: this.uidValue }))
    }

    async getMemberTotal() {
        return JSON.parse(await invoke("get_total_per_member", { sectionUid: this.uidValue }))
    }

    async getGroupTotal() {
        return JSON.parse(await invoke('get_group_sum_calculated_expenses'))
    }

    async getGroupRatioTotal() {
        return JSON.parse(await invoke('get_group_only_sum_calculated_expenses'))
    }

    async expenseListTargetConnected() {
        await this.expenseListLoad()
    }

    async sectionMembersCountTargetConnected() {
        await this.loadSectionMembersCount()
    }

    async sectionAdultsCountTargetConnected() {
        await this.loadSectionAdultsCount()
    }

    async expenseInstanceListTargetConnected() {
        await this.expenseInstanceListLoad()
    }

    async expenseGroupInstanceListTargetConnected() {
        await this.expenseGroupInstanceListLoad()
    }

    async expenseInstanceGroupTotalTargetConnected() {
        await this.expenseInstanceGroupTotalLoad()
    }

    async expenseInstanceTotalTargetConnected() {
        await this.expenseInstanceTotalLoad()
    }

    async expenseInstanceMemberTotalTargetConnected() {
        await this.expenseInstanceMemberTotalLoad()
    }

    async expenseGroupRatioTotalTargetConnected() {
        await this.expenseGroupRatioTotalLoad()
    }

    async expenseGroupRatioTotalLoad() {
        if (GROUP_ID !== this.uidValue) {
            return
        }

        let total = await this.getGroupRatioTotal()
        //TODO change template
        renderElement(this.expenseGroupRatioTotalTarget, await generateFromFilePath('_parts/_components/_matrix_section_total_ratio.html', total))
    }

    async expenseListLoad() {
        let expenseList = await this.getExpenseList()
        renderElement(this.expenseListTarget, await generateFromFilePath('_parts/_components/_matrix_section_expense.html', expenseList))
    }

    async loadSectionMembersCount() {
        this.sectionMembersCountTarget.value = await this.getMembersCount()
        if (GROUP_ID === this.uidValue) {
            this.sectionMembersCountTarget.setAttribute('readonly', 'readonly')
        }
    }

    async loadSectionAdultsCount() {
        this.sectionAdultsCountTarget.value = await this.getAdultsCount()
        if (GROUP_ID === this.uidValue) {
            this.sectionAdultsCountTarget.setAttribute('readonly', 'readonly')
        }
    }

    async expenseInstanceListLoad() {
        let expenseInstanceList = await this.getUsedExpenseList()
        renderElement(this.expenseInstanceListTarget, await generateFromFilePath('_parts/_components/_matrix_section_expense_instance.html', expenseInstanceList))
    }

    async expenseInstanceGroupTotalLoad() {
        if (GROUP_ID === this.uidValue) {
            return
        }

        let total = await this.getGroupTotal()
        renderElement(this.expenseInstanceGroupTotalTarget, await generateFromFilePath('_parts/_components/_matrix_section_global_total.html', total))
    }

    async expenseInstanceTotalLoad() {
        let total = await this.getTotal()
        renderElement(this.expenseInstanceTotalTarget, await generateFromFilePath('_parts/_components/_matrix_section_total.html', total))
    }

    async expenseInstanceMemberTotalLoad() {
        if (GROUP_ID === this.uidValue) {
            return
        }
        let total = await this.getMemberTotal()
        renderElement(this.expenseInstanceMemberTotalTarget, await generateFromFilePath('_parts/_components/_matrix_section_total_per_member.html', total))
    }

    async expenseGroupInstanceListLoad() {
        if (GROUP_ID != this.uidValue) {
            return
        }

        let groupExpenseInstanceList = await this.getGroupUsedExpenseList()
        if (0 === groupExpenseInstanceList.length) {
            this.expenseGroupInstanceListContainerTarget.classList.add('d-none')
            return
        }

        renderElement(this.expenseGroupInstanceListTarget, await generateFromFilePath('_parts/_components/_matrix_section_group_expense_instance.html', groupExpenseInstanceList))
        this.expenseGroupInstanceListContainerTarget.classList.remove('d-none')
    }

    async updateMembersCount(e) {
        if (!this.validateMembersCount()) {
            return
        }
        await invoke("update_members_count", { uid: this.uidValue, membersCount: parseInt(this.sectionMembersCountTarget.value) })
        this.triggerGlobalRefresh()
    }

    async updateAdultsCount(e) {
        if (!this.validateAdultsCount()) {
            return
        }
        await invoke("update_adults_count", { uid: this.uidValue, adultsCount: parseInt(this.sectionAdultsCountTarget.value) })
        this.triggerGlobalRefresh()
    }

    validateMembersCount() {
        return '' !== this.sectionMembersCountTarget.value.trim()
            && !isNaN(this.sectionMembersCountTarget.value)
            && this.sectionMembersCountTarget.value >= 0
    }

    validateAdultsCount() {
        return '' !== this.sectionAdultsCountTarget.value.trim()
            && !isNaN(this.sectionAdultsCountTarget.value)
            && this.sectionAdultsCountTarget.value >= 0
    }

    async triggerGlobalRefresh() {
        await this.matrixOutlet.refreshAllData()
    }

    sectionRefresh() {
        this.loadSectionMembersCount()
        this.loadSectionAdultsCount()
        this.expenseListLoad()
        this.expenseInstanceListLoad()
        this.expenseGroupInstanceListLoad()
        this.expenseInstanceGroupTotalLoad()
        this.expenseInstanceTotalLoad()
        this.expenseInstanceMemberTotalLoad()
        this.expenseGroupRatioTotalLoad()
    }
})

Stimulus.register("matrix-section-expense", class extends Controller {
    static targets = ["count"]
    static outlets = ["matrix-section"]
    static values = {
        uidSection: String,
        uidExpense: String
    }

    async countTargetConnected() {
        let expenseFromInstance = JSON.parse(await invoke("get_section_expense_from_instance", { sectionUid: this.uidSectionValue, expenseUid: this.uidExpenseValue }))
        this.countTarget.innerHTML = expenseFromInstance.length
    }

    async addExpenseInstance(e) {
        await invoke("add_expense_instance", { sectionUid: this.uidSectionValue, expenseId: this.uidExpenseValue })
        this.matrixSectionOutlet.triggerGlobalRefresh()
    }
})

Stimulus.register("matrix-expense-instance", class extends Controller {
    static targets = ["unitPrice", "units", "unitsAdults", "rate", "comments"]
    static outlets = ["matrix-section"]
    static values = {
        uid: String
    }

    async deleteExpenseInstance() {
        this.element.classList.add('table-active')
        if (await confirm("Veux-tu vraiment supprimer cette dépense ?")) {
            await invoke("delete_expense_instance", { uidExpenseInstance: this.uidValue })
            this.matrixSectionOutlet.triggerGlobalRefresh()
        }
        this.element.classList.remove('table-active')
    }

    copyExpenseInstance() {
        invoke("copy_expense_instance", { uidExpenseInstance: this.uidValue })
        this.matrixSectionOutlet.triggerGlobalRefresh()
    }

    async updateExpenseInstance() {
        if (!this.validate()) {
            return
        }

        await invoke("update_expense_instance", {
            uidExpenseInstance: this.uidValue,
            unitPrice: this.unitPriceTarget.value,
            units: this.unitsTarget.value,
            unitsAdults: this.unitsAdultsTarget.value,
            rate: this.rateTarget.value,
            comments: this.commentsTarget.value,
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

    unitsAdultsValid() {
        return '' === this.unitsAdultsTarget.value.trim()
            || !isNaN(this.unitsAdultsTarget.value)
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
            && this.unitsAdultsValid()
            && this.rateValid()
    }
})