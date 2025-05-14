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

function deleteSpecialCharForId(id) {
    const spec = /[^a-zA-Z0-9_]+/g
    return id.replace(spec, "")
}

function getSelector(element) {
    if (!(element instanceof Element)) return

    const path = []
    while (element && Node.ELEMENT_NODE === element.nodeType) {
        let selector = element.nodeName.toLowerCase()
        let sibling = element,
            nth = 1
        while (sibling.previousElementSibling) {
            sibling = sibling.previousElementSibling
            if (sibling.nodeName.toLowerCase() === selector) nth++
        }
        if (nth !== 1) selector += `:nth-of-type(${nth})`
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
        if (targetElement) {
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
    static targets = ['textInput', 'message', 'main', 'links', 'export']
    static classes = ["loading"]

    filePath = ''

    async connect() {
        this.filePathLoaded()
        this.loadHelp()
    }

    async filePathLoaded() {
        this.filePath = await invoke("get_global_file_path")
        if ('' !== this.filePath.trim()) {
            this.linksTarget.classList.remove('d-none')
            this.exportTarget.classList.remove('d-none')
        }
    }

    async resetDisplay() {
        this.loadHelp()
        this.linksTarget.classList.add('d-none')
        this.exportTarget.classList.add('d-none')
    }

    async openFile(e) {
        this.element.classList.add(this.loadingClass)
        const file = await open({
            multiple: false,
            directory: false,
            filters: [{ name: "Longchamp Budget", extensions: ["lb"] }]
        })

        if (file) {
            await this.resetDisplay()
            await invoke("update_db_path", { path: file, eraseIfExists: false })
            await this.filePathLoaded()
        }
        this.element.classList.remove(this.loadingClass)
    }

    async createFile(e) {
        this.element.classList.add(this.loadingClass)
        const file = await save({
            defaultPath: "budget.lb",
            filters: [{ name: "Longchamp Budget", extensions: ["lb"] }]
        })

        if (file) {
            await this.resetDisplay()
            await invoke("update_db_path", { path: file, eraseIfExists: true })
            await this.filePathLoaded()
        }
        this.element.classList.remove(this.loadingClass)
    }

    export() {
        invoke("generate_xls_file")
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

    loadHelp() {
        loadPart('_parts/_windows/_help.html', this.mainTarget)
    }
})


Stimulus.register("section", class extends Controller {
    static targets = ['title', 'color', 'sectionList', 'sectionMembersCount', 'sectionAdultsCount']
    static outlets = ["budget"]

    sectionList = null

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
        this.sectionList = JSON.parse(await invoke("section_list_load"))

        if (!this.sectionList) {
            return
        }

        renderElement(this.sectionListTarget, await generateFromFilePath('_parts/_components/_section-edit-item.html', this.sectionList))
    }

    async dragstart(e) {
        await e.dataTransfer.setData("text/plain", e.target.getAttribute("data-section-edit-uid-value"))
    }

    async dragover(e) {
        await e.preventDefault()
    }

    async drop(e) {
        await e.preventDefault()
        const tr = e.target.closest('tr') ?? e.target,
            uidList = this.sectionList.map((item) => item.uid),
            draggedElementUid = e.dataTransfer.getData("text/plain"),
            sourcePosition = uidList.indexOf(draggedElementUid),
            element = uidList.splice(sourcePosition, 1)[0],
            targetPosition = uidList.indexOf(tr.getAttribute('data-section-edit-uid-value'))

        if (-1 === sourcePosition) {
            return
        }


        uidList.splice(targetPosition, 0, element)
        await invoke("update_section_order", { sectionList: JSON.stringify(uidList) })
        this.sectionListLoad()
    }

    validateTitle() {
        this.titleTarget.classList.remove('invalid')
        if ('' !== this.titleTarget.value.trim()) {
            return true
        }
        this.titleTarget.classList.add('invalid')
        return false
    }

    validateColor() {
        this.colorTarget.classList.remove('invalid')
        if ('' !== this.colorTarget.value.trim()) {
            return true
        }
        this.colorTarget.classList.add('invalid')
        return false
    }

    validateMembers() {
        this.sectionMembersCountTarget.classList.remove('invalid')
        if ('' !== this.sectionMembersCountTarget.value.trim()
            && !isNaN(this.sectionMembersCountTarget.value)
        ) {
            return true
        }
        this.sectionMembersCountTarget.classList.add('invalid')
        return false
    }

    validateAdults() {
        this.sectionAdultsCountTarget.classList.remove('invalid')
        if ('' !== this.sectionAdultsCountTarget.value.trim()
            && !isNaN(this.sectionAdultsCountTarget.value)) {
            return true
        }
        this.sectionAdultsCountTarget.classList.add('invalid')
        return false
    }

    validate() {
        const validateArray = [
            this.validateTitle(),
            this.validateColor(),
            this.validateMembers(),
            this.validateAdults(),
        ]
        return validateArray.filter((item) => item).length === validateArray.length
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

    async update(e) {
        if (!this.validate()) {
            return
        }
        await invoke("update_section", { uid: this.uidValue, title: this.titleTarget.value.trim(), color: this.colorTarget.value.trim(), membersCount: parseInt(this.sectionMembersCountTarget.value), adultsCount: parseInt(this.sectionAdultsCountTarget.value) })
        this.sectionOutlet.sectionListLoad()
    }

    async delete(e) {
        if (await this.isUsed()) {
            alert("Vous ne pouvez pas supprimer cette unité.\nElle est déja utilisée à une dépense.")
            return
        }
        await invoke("delete_section", { uid: this.uidValue })
        this.sectionOutlet.sectionListLoad()
    }

    validateTitle() {
        this.titleTarget.classList.remove('invalid')
        if ('' !== this.titleTarget.value.trim()) {
            return true
        }
        this.titleTarget.classList.add('invalid')
        return false
    }

    validateColor() {
        this.colorTarget.classList.remove('invalid')
        if ('' !== this.colorTarget.value.trim()) {
            return true
        }
        this.colorTarget.classList.add('invalid')
        return false
    }

    validateMembers() {
        this.sectionMembersCountTarget.classList.remove('invalid')
        if ('' !== this.sectionMembersCountTarget.value.trim()
            && !isNaN(this.sectionMembersCountTarget.value)) {
            return true
        }
        this.sectionMembersCountTarget.classList.add('invalid')
        return false
    }

    validateAdults() {
        this.sectionAdultsCountTarget.classList.remove('invalid')
        if ('' !== this.sectionAdultsCountTarget.value.trim()
            && !isNaN(this.sectionAdultsCountTarget.value)) {
            return true
        }
        this.sectionAdultsCountTarget.classList.add('invalid')
        return false
    }

    validate() {
        const validateArray = [
            this.validateTitle(),
            this.validateColor(),
            this.validateMembers(),
            this.validateAdults()
        ]

        return validateArray.filter((item) => item).length === validateArray.length
    }
})

Stimulus.register("expense", class extends Controller {
    static targets = ['title', 'description', 'rate', 'unitPrice', 'expenseList', 'sectionList', 'section']

    usedSectionExpense = null
    associatedSectionExpense = null
    sectionList = null
    expenseList = null

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

        this.expenseList = JSON.parse(await invoke("expense_list_load"))

        if (!this.expenseList) {
            return
        }

        renderElement(this.expenseListTarget, await generateFromFilePath('_parts/_components/_expense-edit-item.html', this.expenseList))
    }

    async dragstart(e) {
        await e.dataTransfer.setData("text/plain", e.target.getAttribute("data-expense-edit-uid-value"))
    }

    async dragover(e) {
        await e.preventDefault()
    }

    async drop(e) {
        await e.preventDefault()
        const tr = e.target.closest('tr') ?? e.target,
            uidList = this.expenseList.map((item) => item.uid),
            draggedElementUid = e.dataTransfer.getData("text/plain"),
            sourcePosition = uidList.indexOf(draggedElementUid),
            element = uidList.splice(sourcePosition, 1)[0],
            targetPosition = uidList.indexOf(tr.getAttribute('data-expense-edit-uid-value'))

        if (-1 === sourcePosition) {
            return
        }


        uidList.splice(targetPosition, 0, element)
        await invoke("update_expense_order", { expenseList: JSON.stringify(uidList) })
        this.expenseListLoad()
    }

    hasAtLeastOneSectionChecked() {
        this.sectionListTarget.classList.remove('invalid')
        if (0 != this.sectionTargets.filter((section) => section.checked).length) {
            return true
        }
        this.sectionListTarget.classList.add('invalid')

        return false
    }

    isRateTargetValid() {
        this.rateTarget.classList.remove('invalid')
        if ('' !== this.rateTarget.value.trim()
            && !isNaN(this.rateTarget.value)
            && parseFloat(this.rateTarget.value) >= 0
            && parseFloat(this.rateTarget.value) <= 100) {
            return true
        }
        this.rateTarget.classList.add('invalid')
        return false
    }

    isTitleTargetValid() {
        this.titleTarget.classList.remove('invalid')
        if ('' !== this.titleTarget.value.trim()) {
            return true
        }
        this.titleTarget.classList.add('invalid')
        return false
    }

    isUnitPriceTargetValid() {
        this.unitPriceTarget.classList.remove('invalid')
        if ('' !== this.unitPriceTarget.value.trim()
            && !isNaN(this.unitPriceTarget.value)
        ) {
            return true
        }
        this.unitPriceTarget.classList.add('invalid')
        return false
    }

    validate() {
        const validateArray = [
            this.isTitleTargetValid(),
            this.isRateTargetValid(),
            this.isUnitPriceTargetValid(),
            this.hasAtLeastOneSectionChecked()
        ]
        return validateArray.filter((item) => item).length === validateArray.length
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
        sectionList = sectionList.map((item) => {
            item.expenseUid = this.uidValue
            return item
        })
        renderElement(this.sectionListTarget, await generateFromFilePath('_parts/_components/_expense-edit-item-sections.html', sectionList))
    }

    async sectionTargetConnected(section) {
        let expenseFromInstance = JSON.parse(await invoke("get_section_expense_from_instance", { sectionUid: section.value, expenseUid: this.uidValue })),
            expenseFromAssociation = JSON.parse(await invoke("get_section_expense_from_association", { sectionUid: section.value, expenseUid: this.uidValue }))

        section.disabled = 0 !== expenseFromInstance.length
        section.checked = 0 !== expenseFromAssociation.length
    }

    async update(e) {
        if (!this.validate()) {
            return
        }

        await invoke("update_expense", { uid: this.uidValue, title: this.titleTarget.value, description: this.descriptionTarget.value, rate: this.rateTarget.value, unitPrice: this.unitPriceTarget.value })
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
            alert("Vous ne pouvez pas supprimer cette dépense.\nElle est déja utilisée par une unité.")
            return
        }
        await invoke("delete_expense", { uid: this.uidValue })
        this.expenseOutlet.expenseListLoad()
    }

    isRateTargetValid() {
        this.rateTarget.classList.remove('invalid')
        if ('' !== this.rateTarget.value.trim()
            && !isNaN(this.rateTarget.value)
            && parseFloat(this.rateTarget.value) >= 0
            && parseFloat(this.rateTarget.value) <= 100) {
            return true
        }
        this.rateTarget.classList.add('invalid')
        return false
    }

    isTitleTargetValid() {
        this.titleTarget.classList.remove('invalid')
        if ('' !== this.titleTarget.value.trim()) {
            return true
        }
        this.titleTarget.classList.add('invalid')
        return false
    }

    isUnitPriceTargetValid() {
        this.unitPriceTarget.classList.remove('invalid')
        if ("" !== this.unitPriceTarget.value.trim()
            && !isNaN(this.unitPriceTarget.value)
        ) {
            return true
        }
        this.unitPriceTarget.classList.add('invalid')
        return false
    }

    hasAtLeastOneSectionChecked() {
        this.sectionListTarget.classList.remove('invalid')
        if (0 != this.sectionTargets.filter((section) => section.checked).length) {
            return true
        }
        this.sectionListTarget.classList.add('invalid')
        return false
    }

    validate() {
        const validateArray = [
            this.isTitleTargetValid(),
            this.isRateTargetValid(),
            this.isUnitPriceTargetValid()
        ]
        return validateArray.filter((item) => item).length === validateArray.length
    }
})

Stimulus.register("matrix", class extends Controller {
    static targets = ['sectionList']
    static outlets = ["matrix-section"]

    async getSectionList() {
        return JSON.parse(await invoke("section_list_load"))
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
        document.getElementById('globalstyle').innerText = await generateFromFilePath('_parts/_components/_matrix_style.css', sectionList, true)
    }

    async refreshAllData() {
        this.matrixSectionOutlets.forEach((outlet) => { outlet.sectionRefresh() })
    }
})

Stimulus.register("matrix-section", class extends Controller {
    static targets = ['expenseList', 'expenseInstanceList', 'expenseGroupRatioTotal', 'expenseGroupInstanceList', 'expenseGroupInstanceListContainer', 'sectionMembersCount', 'sectionAdultsCount', 'expenseInstanceGroupTotal', 'expenseInstanceTotal', 'expenseInstanceMemberTotal', 'groupSumContainer', 'clearLink']
    static outlets = ['matrix']
    static values = {
        uid: String
    }

    expenseInstanceList = null

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

    async groupSumContainerTargetConnected() {
        await this.groupSumContainerLoad()
    }

    async groupSumContainerLoad() {
        if (GROUP_ID !== this.uidValue) {
            return
        }

        this.groupSumContainerTarget.classList.remove('d-none')

        let ratioTotal = await this.getGroupRatioTotal(),
            total = await this.getTotal(),
            groupTotal = await this.getGroupTotal(),
            data = { ratio: ratioTotal.sum_unit, total: total.sum_unit, groupTotal: groupTotal.sum_unit }

        renderElement(this.groupSumContainerTarget, await generateFromFilePath('_parts/_components/_matrix_section_group_sum.html', data))
    }

    async expenseGroupRatioTotalLoad() {
        if (GROUP_ID !== this.uidValue) {
            return
        }

        let total = await this.getGroupRatioTotal()
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
        this.expenseInstanceList = await this.getUsedExpenseList()

        this.expenseInstanceList = this.expenseInstanceList.map((item) => {
            item.uid_expense_instance_escaped = deleteSpecialCharForId(item.uid_expense_instance)
            return item
        })

        renderElement(this.expenseInstanceListTarget, await generateFromFilePath('_parts/_components/_matrix_section_expense_instance.html', this.expenseInstanceList))
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

        groupExpenseInstanceList = groupExpenseInstanceList.map((item) => {
            item.uid_expense_instance_escaped = deleteSpecialCharForId(item.uid_expense_instance)
            return item
        })

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


    async dragstart(e) {
        await e.dataTransfer.setData("text/plain", e.target.getAttribute("data-matrix-expense-instance-uid-value"))
    }

    async dragover(e) {
        await e.preventDefault()
    }

    async drop(e) {
        await e.preventDefault()
        const tr = e.target.closest('tr') ?? e.target,
            uidList = this.expenseInstanceList.map((item) => item.uid_expense_instance),
            draggedElementUid = e.dataTransfer.getData("text/plain"),
            sourcePosition = uidList.indexOf(draggedElementUid),
            element = uidList.splice(sourcePosition, 1)[0],
            targetPosition = uidList.indexOf(tr.getAttribute('data-matrix-expense-instance-uid-value'))

        if (-1 === sourcePosition) {
            return
        }


        uidList.splice(targetPosition, 0, element)
        await invoke("update_expense_instance_order", { expenseInstanceList: JSON.stringify(uidList) })
        await this.triggerGlobalRefresh()
    }

    async reinitFilter(e) {
        await e.preventDefault()
        document.getElementById('tempstyle').innerText = ''
    }

    validateMembersCount() {
        this.sectionMembersCountTarget.classList.remove('invalid')
        if ('' !== this.sectionMembersCountTarget.value.trim()
            && !isNaN(this.sectionMembersCountTarget.value)
            && this.sectionMembersCountTarget.value >= 0) {
            return true
        }
        this.sectionMembersCountTarget.classList.add('invalid')
        return false
    }

    validateAdultsCount() {
        this.sectionAdultsCountTarget.classList.remove('invalid')
        if ('' !== this.sectionAdultsCountTarget.value.trim()
            && !isNaN(this.sectionAdultsCountTarget.value)
            && this.sectionAdultsCountTarget.value >= 0) {
            return true
        }
        this.sectionAdultsCountTarget.classList.add('invalid')
        return false
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
        this.groupSumContainerLoad()
    }
})

Stimulus.register("matrix-section-expense", class extends Controller {
    static targets = ["count"]
    static outlets = ["matrix-section", ""]
    static values = {
        uidSection: String,
        uidExpense: String
    }

    async countTargetConnected() {
        let expenseFromInstance = await invoke("get_section_expense_cnt_from_instance", { sectionUid: this.uidSectionValue, expenseUid: this.uidExpenseValue })
        this.countTarget.innerHTML = expenseFromInstance
    }

    async addExpenseInstance(e) {
        await invoke("add_expense_instance", { sectionUid: this.uidSectionValue, expenseId: this.uidExpenseValue })
        this.matrixSectionOutlet.triggerGlobalRefresh()
    }

    async highlightExpense() {
        const expenseObject = {
            uidExpense: this.uidExpenseValue
        },
            itemStyle = document.getElementById('tempstyle'),
            applyedStyle = await generateFromFilePath('_parts/_components/_matrix_expense_style.css', expenseObject, true)

        itemStyle.innerText = applyedStyle
    }
})

Stimulus.register("matrix-expense-instance", class extends Controller {
    static targets = ["label", "unitPrice", "number", "units", "unitsAdults", "rate", "comments"]
    static outlets = ["matrix-section"]
    static values = {
        uid: String,
        label: String,
        rate: Number,
        expense: String
    }

    async connect() {
        if (100 != this.rateValue) {
            let data = { uid: deleteSpecialCharForId(this.uidValue), label: this.labelValue }
            renderElement(this.labelTarget, await generateFromFilePath('_parts/_components/_matrix_section_expense_instance_label_link.html', data))
        }
    }

    async deleteExpenseInstance() {
        this.element.classList.add('table-active')
        if (await confirm("Voulez-vous vraiment supprimer cette dépense ?")) {
            await invoke("delete_expense_instance", { uidExpenseInstance: this.uidValue })
            this.matrixSectionOutlet.triggerGlobalRefresh()
        }
        this.element.classList.remove('table-active')
    }

    async copyExpenseInstance() {
        await invoke("copy_expense_instance", { uidExpenseInstance: this.uidValue })
        this.matrixSectionOutlet.triggerGlobalRefresh()
    }

    async updateExpenseInstance(e) {
        if (!this.validate()) {
            return
        }

        await invoke("update_expense_instance", {
            uidExpenseInstance: this.uidValue,
            unitPrice: this.unitPriceTarget.value,
            number: this.numberTarget.value,
            units: this.unitsTarget.value,
            unitsAdults: this.unitsAdultsTarget.value,
            rate: this.rateTarget.value,
            comments: this.commentsTarget.value,
        })
        await this.matrixSectionOutlet.triggerGlobalRefresh()
    }


    clickAnchor(e) {
        const targetIdSelector = '#matrix-section-group',
            targetItem = document.querySelector(targetIdSelector)

        if (!targetItem) {
            return
        }

        targetItem.classList.remove('hide')
        targetItem.classList.add('show')
    }

    unitPriceValid() {
        this.unitPriceTarget.classList.remove('invalid')
        if ('' === this.unitPriceTarget.value.trim()
            || !isNaN(this.unitPriceTarget.value)
        ) {
            return true
        }
        this.unitPriceTarget.classList.add('invalid')
        return false
    }

    unitsValid() {
        this.unitsTarget.classList.remove('invalid')
        if ('' === this.unitsTarget.value.trim()
            || !isNaN(this.unitsTarget.value)
        ) {
            return true
        }
        this.unitsTarget.classList.add('invalid')
        return false
    }

    unitsAdultsValid() {
        this.unitsAdultsTarget.classList.remove('invalid')
        if ('' === this.unitsAdultsTarget.value.trim()
            || !isNaN(this.unitsAdultsTarget.value)) {
            return true
        }
        this.unitsAdultsTarget.classList.add('invalid')
        return false
    }

    rateValid() {
        this.rateTarget.classList.remove('invalid')
        if ('' === this.rateTarget.value.trim()
            || (
                !isNaN(this.rateTarget.value)
                && parseInt(this.rateTarget.value) >= 0
                && parseInt(this.rateTarget.value) <= 100
            )) {
            return true
        }
        this.rateTarget.classList.add('invalid')
        return false
    }

    numberValid() {
        this.numberTarget.classList.remove('invalid')
        if ('' !== this.numberTarget.value.trim()
            || !isNaN(this.numberTarget.value)
        ) {
            return true
        }
        this.numberTarget.classList.add('invalid')
        return false
    }

    validate() {
        const validateArray = [
            this.unitPriceValid(),
            this.unitsValid(),
            this.unitsAdultsValid(),
            this.rateValid(),
            this.numberValid()
        ]
        return validateArray.filter((item) => item).length === validateArray.length
    }
})

Stimulus.register("matrix-group-expense-instance", class extends Controller {
    static targets = []
    static outlets = []
    static values = {
        uidSection: String,
        uidExpense: String
    }

    clickAnchor(e) {
        const targetIdSelector = '#matrix-section-' + this.uidSectionValue,
            targetItem = document.querySelector(targetIdSelector)

        if (!targetItem) {
            return
        }

        targetItem.classList.remove('hide')
        targetItem.classList.add('show')
    }
})