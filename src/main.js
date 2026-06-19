/**
 * @file main.js
 * @description Main entry point for the Longchamp Budget application. 
 * Manages UI interactions using Stimulus and communicates with the Rust backend via Tauri.
 */

// System imports from Tauri
const { invoke } = window.__TAURI__.core
const { open, save } = window.__TAURI__.dialog
const { getCurrentWindow } = window.__TAURI__.window

/** @constant {string} GROUP_ID - The identifier for the global group section */
const GROUP_ID = 'group'

// Stimulus JS import
import { Application, Controller } from "/stimulus.min.js"

/** @type {Object.<string, string>} assetPath - Cache for fetched HTML templates */
let assetPath = {}
const softName = "Longchamp Budget"
const currentWindow = getCurrentWindow()

// Set initial window title
currentWindow.setTitle(softName)

/**
 * Disable context menu in production mode to prevent inspection.
 */
const mode = await invoke('get_build_mode')
if ('debug' !== mode) {
    document.addEventListener('contextmenu', (e) => {
        e.preventDefault()
        return false
    }, false)
}

/**
 * Renders a template string by replacing placeholders with data.
 * Supports {% key %} for raw insertion and {{ key }} for escaped insertion by default.
 * 
 * @param {string} templateString - The HTML template string.
 * @param {Object} data - The data object to populate the template.
 * @param {boolean} [raw=false] - If true, {{ key }} will not be escaped.
 * @returns {string} The rendered HTML string.
 */
function renderTemplate(templateString, data, raw = false) {
    return templateString
        .replace(/{%(.*?)%}/g, (match, p1) => data[p1.trim()])
        .replace(/{{(.*?)}}/g, (match, p1) => {
            const key = p1.trim()
            return raw ? data[key] ?? '' : escapeHtmlAttribute(data[key] ?? '')
        })
}

/**
 * Sanitizes a string to be used as a valid HTML ID by removing special characters.
 * 
 * @param {string} id - The original string.
 * @returns {string} The sanitized string.
 */
function deleteSpecialCharForId(id) {
    const spec = /[^a-zA-Z0-9_]+/g
    return id.replace(spec, "")
}

/**
 * Generates a unique CSS selector for a given DOM element.
 * 
 * @param {Element} element - The DOM element.
 * @returns {string|undefined} The CSS selector string.
 */
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

/**
 * Renders HTML content into an element and attempts to restore focus to the previously active element.
 * 
 * @param {HTMLElement} element - The target container.
 * @param {string} content - The HTML content to inject.
 */
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

/**
 * Fetches an HTML template part from the backend assets.
 * Uses a local cache to avoid redundant requests.
 * 
 * @param {string} htmlPart - The path to the HTML asset.
 * @returns {Promise<string>} The template content.
 */
async function fetchPart(htmlPart) {

    if (!assetPath[htmlPart]) {
        assetPath[htmlPart] = await invoke('read_asset', { path: htmlPart })
    }
    return assetPath[htmlPart]
}

/**
 * Loads a template from a file path and renders it with the provided data.
 * Supports both single objects and arrays of data.
 * 
 * @param {string} filePathString - The path to the template file.
 * @param {Object|Array} data - The data to render.
 * @param {boolean} [raw=false] - Whether to skip HTML escaping.
 * @returns {Promise<string>} The rendered HTML.
 */
async function generateFromFilePath(filePathString, data, raw = false) {
    let strPrototype = await fetchPart(filePathString)
    return Array.isArray(data) ?
        data.map((obj) => renderTemplate(strPrototype, obj, raw)).join('') :
        renderTemplate(strPrototype, data, raw)
}

/**
 * Fetches a template and injects it directly into a target element.
 * 
 * @param {string} htmlPart - The path to the HTML asset.
 * @param {HTMLElement} target - The target container.
 */
async function loadPart(htmlPart, target) {
    renderElement(target, await fetchPart(htmlPart))
}

/**
 * Escapes special characters in a string for safe use in HTML attributes.
 * 
 * @param {string|number} str - The string or number to escape.
 * @returns {string} The escaped string.
 */
function escapeHtmlAttribute(str) {
    return str.toString().replace(/["'&<>]/g, (char) => ({ '"': '&quot;', "'": '&#39;', '&': '&amp;', '<': '&lt;', '>': '&gt;' }[char] ?? char))
}

window.Stimulus = Application.start()

/**
 * Budget Controller
 * Main controller for the application shell. Manages file operations and high-level view loading.
 */
Stimulus.register("budget", class extends Controller {
    static targets = ['textInput', 'message', 'main', 'links', 'export']
    static classes = ["loading"]

    /** @type {string} filePath - Path of the currently loaded budget file */
    filePath = ''

    async connect() {
        this.filePathLoaded()
        this.loadHelp()
    }

    /**
     * Checks if a budget file path is already loaded globally and updates the UI accordingly.
     */
    async filePathLoaded() {
        this.filePath = await invoke("get_global_file_path")
        if ('' !== this.filePath.trim()) {
            this.linksTarget.classList.remove('d-none')
            this.exportTarget.classList.remove('d-none')
        }
        if ('' !== this.filePath.trim()) {
            await currentWindow.setTitle(softName + ' - ' + this.filePath.trim())
        }
    }

    /**
     * Resets the display to the initial help state and hides file-specific links.
     */
    async resetDisplay() {
        this.loadHelp()
        document.getElementById('tempstyle').innerText = ''
        this.linksTarget.classList.add('d-none')
        this.exportTarget.classList.add('d-none')
        document.title = ''
        await currentWindow.setTitle(softName)
    }

    /**
     * Opens a file dialog to select an existing .lb budget file.
     */
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

    /**
     * Opens a save dialog to create a new .lb budget file.
     */
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

    /**
     * Triggers the generation of an Excel export file.
     */
    export() {
        invoke("generate_xls_file")
    }

    /**
     * Loads the Expenses management view.
     */
    loadExpenses() {
        document.getElementById('tempstyle').innerText = ''
        loadPart('_parts/_windows/_expenses.html', this.mainTarget)
    }

    /**
     * Loads the Sections management view.
     */
    loadSections() {
        document.getElementById('tempstyle').innerText = ''
        loadPart('_parts/_windows/_sections.html', this.mainTarget)
    }

    /**
     * Loads the Budget Matrix view.
     */
    loadMatrix() {
        document.getElementById('tempstyle').innerText = ''
        loadPart('_parts/_windows/_matrix.html', this.mainTarget)
    }

    /**
     * Loads the Help view.
     */
    loadHelp() {
        document.getElementById('tempstyle').innerText = ''
        loadPart('_parts/_windows/_help.html', this.mainTarget)
    }

    /**
     * Loads the Quote-Part (FQ) management view.
     */
    loadFqs() {
        document.getElementById('tempstyle').innerText = ''
        loadPart('_parts/_windows/_fqs.html', this.mainTarget)
    }
})


/**
 * Section Controller
 * Manages the creation and listing of budget sections (units).
 */
Stimulus.register("section", class extends Controller {
    static targets = ['title', 'color', 'sectionList', 'sectionMembersCount', 'sectionAdultsCount']
    static outlets = ["budget"]

    sectionList = null

    connect() {
    }

    /**
     * Triggered when the section list target is connected to the DOM.
     */
    sectionListTargetConnected(element) {
        this.sectionListLoad()
    }

    usedSectionExpense = null

    /**
     * Fetches expenses that are associated with any section.
     * @returns {Promise<Object>}
     */
    async getUsedSectionExpense() {
        if (null === this.usedSectionExpense) {
            this.usedSectionExpense = JSON.parse(await invoke("get_section_expense_from_expenses_instances"))
        }
        return this.usedSectionExpense
    }

    /**
     * Creates a new section.
     */
    async create(e) {
        e.preventDefault()
        if (!this.validate()) {
            return
        }
        await invoke("insert_new_section", { 
            title: this.titleTarget.value, 
            color: this.colorTarget.value, 
            membersCount: parseInt(this.sectionMembersCountTarget.value), 
            adultsCount: parseInt(this.sectionAdultsCountTarget.value) 
        })
        this.budgetOutlet.loadSections()
    }

    /**
     * Loads the list of sections from the database and renders them.
     */
    async sectionListLoad() {
        this.sectionList = JSON.parse(await invoke("section_list_load"))

        if (!this.sectionList) {
            return
        }

        renderElement(this.sectionListTarget, await generateFromFilePath('_parts/_components/_section-edit-item.html', this.sectionList))
    }

    /**
     * Handles the start of a drag event for reordering sections.
     */
    async dragstart(e) {
        await e.dataTransfer.setData("text/plain", e.target.getAttribute("data-section-edit-uid-value"))
    }

    /**
     * Allows dropping on the target.
     */
    async dragover(e) {
        await e.preventDefault()
    }

    /**
     * Handles the drop event to reorder sections.
     */
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

    // Validation methods
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

    /**
     * Validates all fields in the section creation form.
     * @returns {boolean}
     */
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

/**
 * Section Edit Controller
 * Manages the editing and deletion of an individual section.
 */
Stimulus.register("section-edit", class extends Controller {
    static targets = ['title', 'color', 'delete', 'sectionMembersCount', 'sectionAdultsCount']
    static outlets = ["section"]
    static values = {
        uid: String
    }

    /**
     * Checks if the section is currently used in any expense instance.
     * @returns {Promise<boolean>}
     */
    async isUsed() {
        let expenseList = JSON.parse(await invoke("get_section_expense_from_expenses_instances_and_section", { sectionUid: this.uidValue })) ?? []
        return 0 !== expenseList.length
    }

    /**
     * Disables the delete button if the section is the group section or is in use.
     */
    async deleteTargetConnected() {
        this.deleteTarget.disabled = this.uidValue == GROUP_ID || await this.isUsed()
    }

    /**
     * Prevents editing member count for the global group section.
     */
    sectionMembersCountTargetConnected() {
        if (GROUP_ID === this.uidValue) {
            this.sectionMembersCountTarget.setAttribute('readonly', 'readonly')
        }
    }

    /**
     * Prevents editing adult count for the global group section.
     */
	sectionAdultsCountTargetConnected() {
		if (GROUP_ID === this.uidValue) {
			this.sectionAdultsCountTarget.setAttribute('readonly', 'readonly')
		}
	}

    /**
     * Updates the section data in the database.
     */
    async update(e) {
        if (!this.validate()) {
            return
        }
        await invoke("update_section", { 
            uid: this.uidValue, 
            title: this.titleTarget.value.trim(), 
            color: this.colorTarget.value.trim(), 
            membersCount: parseInt(this.sectionMembersCountTarget.value), 
            adultsCount: parseInt(this.sectionAdultsCountTarget.value) 
        })
        this.sectionOutlet.sectionListLoad()
    }

    /**
     * Deletes the section if it is not in use.
     */
    async delete(e) {
		if (await this.isUsed()) {
			alert("Vous ne pouvez pas supprimer cette unité.\nElle est déja reliée à une dépense.")
            return
        }
        await invoke("delete_section", { uid: this.uidValue })
        this.sectionOutlet.sectionListLoad()
    }

    // Validation methods
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

    /**
     * Validates all fields in the section edit form.
     * @returns {boolean}
     */
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

/**
 * Expense Controller
 * Manages the definition and creation of different expense types.
 */
Stimulus.register("expense", class extends Controller {
    static targets = ['title', 'description', 'rate', 'unitPrice', 'expenseList', 'sectionList', 'section']

    usedSectionExpense = null
    associatedSectionExpense = null
    sectionList = null
    expenseList = null

    /**
     * Fetches expense instances assigned to sections.
     * @returns {Promise<Object>}
     */
    async getUsedSectionExpense() {
        if (null === this.usedSectionExpense) {
            this.usedSectionExpense = JSON.parse(await invoke("get_section_expense_from_expenses_instances"))
        }
        return this.usedSectionExpense
    }

    /**
     * Fetches the association between expense types and sections.
     * @returns {Promise<Object>}
     */
    async getAssociatedSectionExpense() {
        if (null === this.associatedSectionExpense) {
            this.associatedSectionExpense = JSON.parse(await invoke("get_section_expense"))
        }
        return this.associatedSectionExpense
    }

    /**
     * Loads the list of available sections.
     * @returns {Promise<Array>}
     */
    async getSectionList() {
        if (null === this.sectionList) {
            this.sectionList = JSON.parse(await invoke("section_list_load"))
        }
        return this.sectionList
    }

    /**
     * Triggered when the expense list target is connected.
     */
    expenseListTargetConnected(element) {
        this.expenseListLoad()
    }

    /**
     * Creates a new expense definition and associates it with selected sections.
     */
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

        // Reset local caches and form
        this.associatedSectionExpense = null
        this.usedSectionExpense = null

        this.titleTarget.value = ''
        this.descriptionTarget.value = ''
        this.rateTarget.value = 100
        this.unitPriceTarget.value = ''
        this.sectionTargets.forEach((section) => section.checked = false)

        this.expenseListLoad()
    }

    /**
     * Renders the list of sections with checkboxes for selection during expense creation.
     */
    async sectionListTargetConnected(element) {
        const sectionList = await this.getSectionList()
        renderElement(element, await generateFromFilePath('_parts/_components/_expense-create-item-sections.html', sectionList))
    }

    /**
     * Loads and renders the list of all expense definitions.
     */
    async expenseListLoad() {

        this.expenseList = JSON.parse(await invoke("expense_list_load"))

        if (!this.expenseList) {
            return
        }

        renderElement(this.expenseListTarget, await generateFromFilePath('_parts/_components/_expense-edit-item.html', this.expenseList))
    }

    /**
     * Handles drag start for reordering expense definitions.
     */
    async dragstart(e) {
        await e.dataTransfer.setData("text/plain", e.target.getAttribute("data-expense-edit-uid-value"))
    }

    /**
     * Allows drag over for reordering.
     */
    async dragover(e) {
        await e.preventDefault()
    }

    /**
     * Handles drop for reordering expense definitions.
     */
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

    /**
     * Compacts the display by applying a specific CSS style.
     */
    async compact(e) {
        e.preventDefault()
        document.getElementById('tempstyle').innerText = await generateFromFilePath('_parts/_components/_expense_style.css', {}, true)
    }

    /**
     * Restores normal display by removing the compact CSS style.
     */
    async decompact(e) {
        e.preventDefault()
        document.getElementById('tempstyle').innerText = ''
    }

    // Validation methods
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

    /**
     * Validates the whole expense creation form.
     * @returns {boolean}
     */
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

/**
 * Expense Edit Controller
 * Manages the editing of an individual expense definition.
 */
Stimulus.register("expense-edit", class extends Controller {
    static targets = ['title', 'description', 'rate', 'unitPrice', 'sectionList', 'section', 'delete']
    static outlets = ["expense"]
    static values = {
        uid: String
    }

    /**
     * Checks if this expense type is currently used in any section's matrix.
     * @returns {Promise<boolean>}
     */
    async isUsed() {
        let expenseFromInstance = JSON.parse(await invoke("get_section_expense_from_instances_by_expense", { expenseUid: this.uidValue }))
        return 0 !== expenseFromInstance.length
    }

    /**
     * Renders the section association list for this expense.
     */
    async sectionListTargetConnected() {
        let sectionList = JSON.parse(await invoke("section_list_load"))
        sectionList = sectionList.map((item) => {
            item.expenseUid = this.uidValue
            return item
        })
        renderElement(this.sectionListTarget, await generateFromFilePath('_parts/_components/_expense-edit-item-sections.html', sectionList))
    }

    /**
     * Configures individual section checkboxes based on usage and current association.
     */
    async sectionTargetConnected(section) {
        let expenseFromInstance = JSON.parse(await invoke("get_section_expense_from_instance", { sectionUid: section.value, expenseUid: this.uidValue })),
            expenseFromAssociation = JSON.parse(await invoke("get_section_expense_from_association", { sectionUid: section.value, expenseUid: this.uidValue }))

        section.disabled = 0 !== expenseFromInstance.length
        section.checked = 0 !== expenseFromAssociation.length
    }

    /**
     * Updates the expense definition.
     */
    async update(e) {
        if (!this.validate()) {
            return
        }

        await invoke("update_expense", { uid: this.uidValue, title: this.titleTarget.value, description: this.descriptionTarget.value, rate: this.rateTarget.value, unitPrice: this.unitPriceTarget.value })
    }

    /**
     * Disables delete button if the expense is in use.
     */
    async deleteTargetConnected(element) {
        element.disabled = await this.isUsed()
    }

    /**
     * Updates the association between this expense and sections.
     */
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

    /**
     * Deletes the expense definition if not in use.
     */
    async delete(e) {
		if (await this.isUsed()) {
			alert("Vous ne pouvez pas supprimer cette dépense.\nElle est déja au budget d'une unité.")
            return
        }
        await invoke("delete_expense", { uid: this.uidValue })
        this.expenseOutlet.expenseListLoad()
    }

    // Validation methods
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

    /**
     * Validates the edit form fields.
     */
    validate() {
        const validateArray = [
            this.isTitleTargetValid(),
            this.isRateTargetValid(),
            this.isUnitPriceTargetValid()
        ]
        return validateArray.filter((item) => item).length === validateArray.length
    }
})

/**
 * Matrix Controller
 * Manages the top-level matrix view, coordinating multiple matrix-section controllers.
 */
Stimulus.register("matrix", class extends Controller {
    static targets = ['sectionList']
    static outlets = ["matrix-section"]

    /**
     * Loads the list of sections.
     * @returns {Promise<Array>}
     */
    async getSectionList() {
        return JSON.parse(await invoke("section_list_load"))
    }

    /**
     * Triggered when the section list target is connected.
     */
    sectionListTargetConnected(element) {
        this.sectionListLoad()
    }

    /**
     * Renders all sections in the matrix view and applies global matrix styles.
     */
    async sectionListLoad() {
        let sectionList = await this.getSectionList()

        if (!sectionList) {
            return
        }

        renderElement(this.sectionListTarget, await generateFromFilePath('_parts/_components/_matrix_section.html', sectionList))
        document.getElementById('globalstyle').innerText = await generateFromFilePath('_parts/_components/_matrix_style.css', sectionList, true)
    }

    /**
     * Refreshes data for all section outlets.
     */
    async refreshAllData() {
        this.matrixSectionOutlets.forEach((outlet) => { outlet.sectionRefresh() })
    }
})

/**
 * Matrix Section Controller
 * Manages the specific view and calculations for a single section within the matrix.
 */
Stimulus.register("matrix-section", class extends Controller {
    static targets = ['expenseList', 'expenseInstanceList', 'expenseGroupRatioTotal', 'expenseGroupInstanceList', 'expenseGroupInstanceListContainer', 'sectionMembersCount', 'sectionAdultsCount', 'expenseInstanceGroupTotal', 'expenseInstanceTotal', 'expenseInstanceMemberTotal', 'groupSumContainer', 'clearLink', 'fqMatrix']
    static outlets = ['matrix']
    static values = {
        uid: String
    }

    expenseInstanceList = null

    // Data fetching methods
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

    // Target connection handlers
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

    async fqMatrixTargetConnected() {
        await this.fqMatrixLoad()
    }

    /**
     * Loads and renders the Quote-Part (FQ) matrix for the section.
     */
    async fqMatrixLoad() {
        // Don't display fqMatrix if no expense instance exists
        let expenseInstanceList = await this.getUsedExpenseList(),
            cond = 0 === expenseInstanceList.length

        // Specific check for global group
        if (GROUP_ID === this.uidValue) {
            let groupExpenseInstanceList = await this.getGroupUsedExpenseList()
            cond = cond && 0 === groupExpenseInstanceList.length
        }

        if (cond) {
            renderElement(this.fqMatrixTarget, '')
            return
        }

        let fqMatrix = JSON.parse(await invoke('get_fqs_calculated_by_section', { sectionUid: this.uidValue }))

        if (0 === fqMatrix.length) {
            return
        }

        let headTemplate = '_parts/_components/_matrix_section_fq_head.html',
            bodyTemplate = '_parts/_components/_matrix_section_fq_body.html',
            tableTemplate = '_parts/_components/_matrix_section_fq_table.html'

        if (GROUP_ID === this.uidValue) {
            headTemplate = '_parts/_components/_matrix_section_fq_head_group.html'
            bodyTemplate = '_parts/_components/_matrix_section_fq_body_group.html'
            tableTemplate = '_parts/_components/_matrix_section_fq_table_group.html'
        }

        let fqHead = fqMatrix[0],
            fqHeadHtmlContent = await generateFromFilePath(headTemplate, fqHead),
            fqBodyHtmlContent = await generateFromFilePath(bodyTemplate, fqMatrix),
            fqTableHtmlContent = await generateFromFilePath(tableTemplate, { fqBodyHtmlContent: fqBodyHtmlContent })

        renderElement(this.fqMatrixTarget, fqHeadHtmlContent + fqTableHtmlContent)
    }

    /**
     * Loads and renders the global group sum container (only for group section).
     */
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

    /**
     * Loads and renders the group ratio total (only for group section).
     */
    async expenseGroupRatioTotalLoad() {
        if (GROUP_ID !== this.uidValue) {
            return
        }

        let total = await this.getGroupRatioTotal()
        renderElement(this.expenseGroupRatioTotalTarget, await generateFromFilePath('_parts/_components/_matrix_section_total_ratio.html', total))
    }

    /**
     * Loads and renders the list of expenses associated with this section.
     */
    async expenseListLoad() {
        let expenseList = await this.getExpenseList()
        renderElement(this.expenseListTarget, await generateFromFilePath('_parts/_components/_matrix_section_expense.html', expenseList))
    }

    /**
     * Loads the member count for the section.
     */
    async loadSectionMembersCount() {
        this.sectionMembersCountTarget.value = await this.getMembersCount()
        if (GROUP_ID === this.uidValue) {
            this.sectionMembersCountTarget.setAttribute('readonly', 'readonly')
        }
    }

    /**
     * Loads the adult count for the section.
     */
    async loadSectionAdultsCount() {
        this.sectionAdultsCountTarget.value = await this.getAdultsCount()
        if (GROUP_ID === this.uidValue) {
            this.sectionAdultsCountTarget.setAttribute('readonly', 'readonly')
        }
    }

    /**
     * Loads and renders the active expense instances for this section.
     */
    async expenseInstanceListLoad() {
        this.expenseInstanceList = await this.getUsedExpenseList()

        this.expenseInstanceList = this.expenseInstanceList.map((item) => {
            item.uid_expense_instance_escaped = deleteSpecialCharForId(item.uid_expense_instance)
            return item
        })

        renderElement(this.expenseInstanceListTarget, await generateFromFilePath('_parts/_components/_matrix_section_expense_instance.html', this.expenseInstanceList))
    }

    /**
     * Loads and renders the global group total within a section's matrix.
     */
    async expenseInstanceGroupTotalLoad() {
        if (GROUP_ID === this.uidValue) {
            return
        }

        let total = await this.getGroupTotal()
        renderElement(this.expenseInstanceGroupTotalTarget, await generateFromFilePath('_parts/_components/_matrix_section_global_total.html', total))
    }

    /**
     * Loads and renders the total for this section.
     */
    async expenseInstanceTotalLoad() {
        let total = await this.getTotal()
        renderElement(this.expenseInstanceTotalTarget, await generateFromFilePath('_parts/_components/_matrix_section_total.html', total))
    }

    /**
     * Loads and renders the per-member total for this section.
     */
    async expenseInstanceMemberTotalLoad() {
        if (GROUP_ID === this.uidValue) {
            return
        }
        let total = await this.getMemberTotal()
        renderElement(this.expenseInstanceMemberTotalTarget, await generateFromFilePath('_parts/_components/_matrix_section_total_per_member.html', total))
    }

    /**
     * Loads and renders global group expense instances (only for group section).
     */
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

    /**
     * Updates the member count and triggers a global refresh.
     */
    async updateMembersCount(e) {
        if (!this.validateMembersCount()) {
            return
        }
        await invoke("update_members_count", { uid: this.uidValue, membersCount: parseInt(this.sectionMembersCountTarget.value) })
        this.triggerGlobalRefresh()
    }

    /**
     * Updates the adult count and triggers a global refresh.
     */
    async updateAdultsCount(e) {
        if (!this.validateAdultsCount()) {
            return
        }
        await invoke("update_adults_count", { uid: this.uidValue, adultsCount: parseInt(this.sectionAdultsCountTarget.value) })
        this.triggerGlobalRefresh()
    }


    /**
     * Handles drag start for reordering expense instances.
     */
    async dragstart(e) {
        await e.dataTransfer.setData("text/plain", e.target.getAttribute("data-matrix-expense-instance-uid-value"))
    }

    /**
     * Allows drag over.
     */
    async dragover(e) {
        await e.preventDefault()
    }

    /**
     * Handles drop for reordering expense instances.
     */
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

    /**
     * Clears specific styles (reinitializes filters).
     */
    async reinitFilter(e) {
        await e.preventDefault()
        document.getElementById('tempstyle').innerText = ''
    }

    // Validation methods
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

    /**
     * Triggers a refresh of all data across all sections via the matrix outlet.
     */
    async triggerGlobalRefresh() {
        await this.matrixOutlet.refreshAllData()
    }

    /**
     * Refreshes all components of this section's matrix view.
     */
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
        this.fqMatrixLoad()
    }
})

/**
 * Matrix Section Expense Controller
 * Manages the interaction for adding a new expense instance to a section.
 */
Stimulus.register("matrix-section-expense", class extends Controller {
    static targets = ["count"]
    static outlets = ["matrix-section", ""]
    static values = {
        uidSection: String,
        uidExpense: String
    }

    /**
     * Loads the current count of instances for this expense in this section.
     */
    async countTargetConnected() {
        let expenseFromInstance = await invoke("get_section_expense_cnt_from_instance", { sectionUid: this.uidSectionValue, expenseUid: this.uidExpenseValue })
        this.countTarget.innerHTML = expenseFromInstance
    }

    /**
     * Adds a new instance of this expense to the current section.
     */
    async addExpenseInstance(e) {
        await invoke("add_expense_instance", { sectionUid: this.uidSectionValue, expenseId: this.uidExpenseValue })
        this.matrixSectionOutlet.triggerGlobalRefresh()
    }

    /**
     * Highlights the current expense across the matrix by applying a dynamic CSS rule.
     */
    async highlightExpense() {
        const expenseObject = {
            uidExpense: this.uidExpenseValue
        },
            itemStyle = document.getElementById('tempstyle'),
            applyedStyle = await generateFromFilePath('_parts/_components/_matrix_expense_style.css', expenseObject, true)

        itemStyle.innerText = applyedStyle
    }
})

/**
 * Matrix Expense Instance Controller
 * Manages the lifecycle and updates of an individual expense instance in the matrix.
 */
Stimulus.register("matrix-expense-instance", class extends Controller {
    static targets = ["label", "unitPrice", "number", "units", "unitsAdults", "rate", "comments"]
    static outlets = ["matrix-section"]
    static values = {
        uid: String,
        label: String,
        rate: Number,
        expense: String
    }

    /**
     * If the rate is not 100%, renders a link to the group section for reference.
     */
    async connect() {
        if (100 != this.rateValue) {
            let data = { uid: deleteSpecialCharForId(this.uidValue), label: this.labelValue }
            renderElement(this.labelTarget, await generateFromFilePath('_parts/_components/_matrix_section_expense_instance_label_link.html', data))
        }
    }

    /**
     * Deletes the expense instance after confirmation.
     */
    async deleteExpenseInstance() {
        this.element.classList.add('table-active')
		if (await confirm("Êtes vous sûr de vouloir supprimer cette dépense ?")) {
            await invoke("delete_expense_instance", { uidExpenseInstance: this.uidValue })
            this.matrixSectionOutlet.triggerGlobalRefresh()
        }
        this.element.classList.remove('table-active')
    }

    /**
     * Creates a duplicate of the expense instance.
     */
    async copyExpenseInstance() {
        await invoke("copy_expense_instance", { uidExpenseInstance: this.uidValue })
        this.matrixSectionOutlet.triggerGlobalRefresh()
    }

    /**
     * Saves updates to the expense instance to the database.
     */
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

    /**
     * Smoothly scrolls to the group section when the anchor link is clicked.
     */
    clickAnchor(e) {
        const targetIdSelector = '#matrix-section-group',
            targetItem = document.querySelector(targetIdSelector)

        if (!targetItem) {
            return
        }

        targetItem.classList.remove('hide')
        targetItem.classList.add('show')
    }

    // Validation methods
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

    /**
     * Validates all input fields for the expense instance.
     */
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

/**
 * Matrix Group Expense Instance Controller
 * Handles interactions for expense instances specifically belonging to the group section.
 */
Stimulus.register("matrix-group-expense-instance", class extends Controller {
    static targets = []
    static outlets = []
    static values = {
        uidSection: String,
        uidExpense: String
    }

    /**
     * Scrolls to the corresponding section matrix when an anchor link is clicked.
     */
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

/**
 * FQ Controller
 * Manages the creation and configuration of Quote-Part (FQ) types.
 */
Stimulus.register("fq", class extends Controller {
    static targets = ['title', 'fqList', 'coeff', 'nationalContribution', 'onlineCommissionRate', 'onlineCommissionFees', 'sectionList']
    static outlets = ["budget"]

    fqList = null

    /**
     * Triggered when the FQ list target is connected.
     */
    fqListTargetConnected(element) {
        this.fqListLoad()
    }

    /**
     * Triggered when the section list target is connected.
     */
    sectionListTargetConnected(element) {
        this.sectionListLoad()
    }

    /**
     * Creates a new FQ type.
     */
    async create(e) {
        e.preventDefault()
        if (!this.validate()) {
            return
        }
        await invoke("insert_new_fq", { 
            title: this.titleTarget.value, 
            coeff: this.coeffTarget.value, 
            nationalContribution: this.nationalContributionTarget.value, 
            onlineCommissionRate: this.onlineCommissionRateTarget.value, 
            onlineCommissionFees: this.onlineCommissionFeesTarget.value 
        })
        await this.globalRefresh()

        // Reset form
        this.titleTarget.value = ''
        this.coeffTarget.value = 1
        this.nationalContributionTarget.value = 0
        this.onlineCommissionFeesTarget.value = 0
        this.onlineCommissionRateTarget.value = 0
    }

    /**
     * Loads and renders the list of all defined FQ types.
     */
    async fqListLoad() {
        this.fqList = JSON.parse(await invoke("fq_list_load"))
        renderElement(this.fqListTarget, await generateFromFilePath('_parts/_components/_fq-edit-item.html', this.fqList))
    }

    /**
     * Refreshes both FQ types and section-specific FQ assignments.
     */
    async globalRefresh() {
        await this.fqListLoad()
        await this.sectionListLoad()
    }

    /**
     * Loads FQ data specifically for a given section.
     * @param {Object} section - The section object.
     * @returns {Promise<Object>} The updated section object.
     */
    async loadFqForSection(section) {
        let fqList = JSON.parse(await invoke("fq_section_list_load", { sectionUid: section.uid }))
        fqList = fqList.map((x) => {
            x.section_members_count = section.members_count
            return x
        })
        section.fqContent = await generateFromFilePath('_parts/_components/_fq-section-fq.html', fqList, true)
        section.membersFqCount = await invoke("get_members_fq_count_by_section", { sectionUid: section.uid })
        return section
    }

    /**
     * Loads and renders the list of sections with their associated FQ data.
     */
    async sectionListLoad() {
        let sectionList = JSON.parse(await invoke("section_list_load"))
        for (let i = 0, j = sectionList.length; i < j; i++) {
            sectionList[i] = await this.loadFqForSection(sectionList[i])
        }

        renderElement(this.sectionListTarget, await generateFromFilePath('_parts/_components/_fq-section.html', sectionList))
    }

    /**
     * Handles drag start for reordering FQ types.
     */
    async dragstart(e) {
        await e.dataTransfer.setData("text/plain", e.target.getAttribute("data-fq-edit-uid-value"))
    }

    /**
     * Allows drag over.
     */
    async dragover(e) {
        await e.preventDefault()
    }

    /**
     * Handles drop for reordering FQ types.
     */
    async drop(e) {
        await e.preventDefault()
        const tr = e.target.closest('tr') ?? e.target,
            uidList = this.fqList.map((item) => item.uid),
            draggedElementUid = e.dataTransfer.getData("text/plain"),
            sourcePosition = uidList.indexOf(draggedElementUid),
            element = uidList.splice(sourcePosition, 1)[0],
            targetPosition = uidList.indexOf(tr.getAttribute('data-fq-edit-uid-value'))

        if (-1 === sourcePosition) {
            return
        }

        uidList.splice(targetPosition, 0, element)
        await invoke("update_fq_order", { fqList: JSON.stringify(uidList) })
        await this.globalRefresh()
    }

    // Validation methods
    validateTitle() {
        this.titleTarget.classList.remove('invalid')
        if ('' !== this.titleTarget.value.trim()) {
            return true
        }
        this.titleTarget.classList.add('invalid')
        return false
    }

    validateCoeff() {
        this.coeffTarget.classList.remove('invalid')
        if ('' !== this.coeffTarget.value.trim()
            && !isNaN(this.coeffTarget.value)
        ) {
            return true
        }
        this.coeffTarget.classList.add('invalid')
        return false
    }

    validateNationalContribution() {
        this.nationalContributionTarget.classList.remove('invalid')
        if ('' !== this.nationalContributionTarget.value.trim()
            && !isNaN(this.nationalContributionTarget.value)) {
            return true
        }
        this.nationalContributionTarget.classList.add('invalid')
        return false
    }

    validateOnlineCommissionRate() {
        this.onlineCommissionRateTarget.classList.remove('invalid')
        if ('' !== this.onlineCommissionRateTarget.value.trim()
            && !isNaN(this.onlineCommissionRateTarget.value)) {
            return true
        }
        this.onlineCommissionRateTarget.classList.add('invalid')
        return false
    }

    validateOnlineCommissionFees() {
        this.onlineCommissionFeesTarget.classList.remove('invalid')
        if ('' !== this.onlineCommissionFeesTarget.value.trim()
            && !isNaN(this.onlineCommissionFeesTarget.value)) {
            return true
        }
        this.onlineCommissionFeesTarget.classList.add('invalid')
        return false
    }

    /**
     * Validates all fields in the FQ creation form.
     */
    validate() {
        const validateArray = [
            this.validateTitle(),
            this.validateCoeff(),
            this.validateNationalContribution(),
            this.validateOnlineCommissionRate(),
            this.validateOnlineCommissionFees(),
        ]
        return validateArray.filter((item) => item).length === validateArray.length
    }
})

/**
 * FQ Edit Controller
 * Manages the editing of an individual FQ type.
 */
Stimulus.register("fq-edit", class extends Controller {
    static targets = ['title', 'coeff', 'nationalContribution', 'onlineCommissionRate', 'onlineCommissionFees']
    static outlets = ["fq"]
    static values = {
        uid: String
    }

    /**
     * Updates the FQ type definition.
     */
    async update(e) {
        if (!this.validate()) {
            return
        }
        await invoke("update_fq", { 
            uid: this.uidValue, 
            title: this.titleTarget.value.trim(), 
            coeff: this.coeffTarget.value, 
            nationalContribution: this.nationalContributionTarget.value, 
            onlineCommissionRate: this.onlineCommissionRateTarget.value, 
            onlineCommissionFees: this.onlineCommissionFeesTarget.value 
        })
        await this.fqOutlet.globalRefresh()
    }

    /**
     * Deletes the FQ type definition.
     */
    async delete(e) {
        await invoke("delete_fq", { uid: this.uidValue })
        await this.fqOutlet.globalRefresh()
    }

    // Validation methods
    validateTitle() {
        this.titleTarget.classList.remove('invalid')
        if ('' !== this.titleTarget.value.trim()) {
            return true
        }
        this.titleTarget.classList.add('invalid')
        return false
    }

    validateCoeff() {
        this.coeffTarget.classList.remove('invalid')
        if ('' !== this.coeffTarget.value.trim()
            && !isNaN(this.coeffTarget.value)) {
            return true
        }
        this.coeffTarget.classList.add('invalid')
        return false
    }

    validateNationalContribution() {
        this.nationalContributionTarget.classList.remove('invalid')
        if ('' !== this.nationalContributionTarget.value.trim()
            && !isNaN(this.nationalContributionTarget.value)) {
            return true
        }
        this.nationalContributionTarget.classList.add('invalid')
        return false
    }

    validateOnlineCommissionRate() {
        this.onlineCommissionRateTarget.classList.remove('invalid')
        if ('' !== this.onlineCommissionRateTarget.value.trim()
            && !isNaN(this.onlineCommissionRateTarget.value)) {
            return true
        }
        this.onlineCommissionRateTarget.classList.add('invalid')
        return false
    }

    validateOnlineCommissionFees() {
        this.onlineCommissionFeesTarget.classList.remove('invalid')
        if ('' !== this.onlineCommissionFeesTarget.value.trim()
            && !isNaN(this.onlineCommissionFeesTarget.value)) {
            return true
        }
        this.onlineCommissionFeesTarget.classList.add('invalid')
        return false
    }

    /**
     * Validates the edit form fields for an FQ type.
     */
    validate() {
        const validateArray = [
            this.validateTitle(),
            this.validateCoeff(),
            this.validateNationalContribution(),
            this.validateOnlineCommissionRate(),
            this.validateOnlineCommissionFees()
        ]

        return validateArray.filter((item) => item).length === validateArray.length
    }
})

/**
 * FQ Section Controller
 * Manages FQ-specific settings for a given section.
 */
Stimulus.register("fq-section", class extends Controller {
    static targets = ['fqSectionList', 'membersCount']
    static outlets = ["fq"]
    static values = {
        sectionUid: String,
    }

    /**
     * Configures the member count input; read-only for the global group.
     */
    async membersCountTargetConnected(element) {
        this.membersCountTarget.readOnly = GROUP_ID === this.sectionUidValue
    }

    /**
     * Updates the section's total member count.
     */
    async updateMembersCount() {
        if (!this.membersCountValid()) {
            return
        }

        await invoke("update_members_count", { uid: this.sectionUidValue, membersCount: parseInt(this.membersCountTarget.value) })
        await this.fqOutlet.globalRefresh()
    }

    // Validation
    membersCountValid() {
        this.membersCountTarget.classList.remove('invalid')
        if ('' === this.membersCountTarget.value.trim()
            || !isNaN(this.membersCountTarget.value)
        ) {
            return true
        }
        this.membersCountTarget.classList.add('invalid')
        return false
    }
})

/**
 * FQ Section FQ Edit Controller
 * Manages the specific member count for an FQ type within a section.
 */
Stimulus.register("fq-section-fq-edit", class extends Controller {
    static targets = ['membersCount']
    static outlets = ["fq"]
    static values = {
        membersCount: Number,
        uidFq: String,
        uidSection: String
    }

    /**
     * Configures the member count input; read-only for the global group.
     */
    async membersCountTargetConnected(element) {
        this.membersCountTarget.readOnly = GROUP_ID === this.uidSectionValue
    }

    // Validation
    validateMembersCount() {
        this.membersCountTarget.classList.remove('invalid')
        if ('' === this.membersCountTarget.value.trim()
            || !isNaN(this.membersCountTarget.value)
        ) {
            return true
        }
        this.membersCountTarget.classList.add('invalid')
        return false
    }

    /**
     * Updates the specific member count for this FQ assignment.
     */
    async update(e) {
        if (!this.validateMembersCount()) {
            return
        }

        await invoke("update_fq_section_members_count", { sectionUid: this.uidSectionValue, fqUid: this.uidFqValue, membersCount: parseInt(this.membersCountTarget.value) })
        await this.fqOutlet.globalRefresh()
    }
})

/**
 * FQ Members Control Controller
 * Displays a warning if the sum of FQ member assignments does not match the section total.
 */
Stimulus.register("fq-members-control", class extends Controller {
    static targets = ['displayMessage']
    static values = {
        sectionMembersCount: Number,
        sectionMembersFqCount: Number
    }

    /**
     * Checks for discrepancy and displays the message if needed.
     */
    async displayMessageTargetConnected(element) {
        if (parseInt(this.sectionMembersCountValue) !== parseInt(this.sectionMembersFqCountValue)) {
            element.classList.remove('d-none')
        }
    }
})

/**
 * Search Controller
 * Provides generic search/filtering functionality for elements marked as searchable.
 */
Stimulus.register("search", class extends Controller {
    static targets = ["input", "searchable"]
    static values = {
        delay: { type: Number, default: 300 }
    }

    connect() {
        this.timeout = null
    }

    disconnect() {
        if (this.timeout) {
            clearTimeout(this.timeout)
        }
    }

    /**
     * Focuses the search input field.
     */
    setFocus() {
        this.inputTarget.focus()
    }

    /**
     * Triggered when a searchable element is added to the DOM.
     * Ensures initial search state is applied.
     */
    async searchableTargetConnected(element) {
        this.searchInContainerAndShow(element)
    }

    /**
     * Debounced search trigger.
     */
    search() {
        clearTimeout(this.timeout)
        this.timeout = setTimeout(() => { this.performSearch() }, this.delayValue)
    }

    /**
     * Iterates through all searchable targets and applies the search filter.
     */
    performSearch() {
        this.searchableTargets.forEach(container => { this.searchInContainerAndShow(container) })
    }

    /**
     * Searches for the query string within a specific container.
     * Checks text content of various elements (inputs, textareas, spans, etc.).
     * 
     * @param {HTMLElement} container - The container to search within.
     */
    searchInContainerAndShow(container) {
        const query = this.inputTarget.value.toLowerCase().trim()

        if ('' !== query) {
            const isNumeric = new RegExp('^[\\+\\-]?[0-9\\.,]+$', 'gmi').test(query)
            for (const element of container.querySelectorAll('input[type="text"], input[type="number"], input[type="email"], textarea, td, span, div, p, a')) {
                const textContent = (-1 !== ["INPUT", "TEXTAREA"].indexOf(element.tagName)) ?
                    (isNumeric && '' === element.value.toString()) ? element.getAttribute('placeholder') : element.value
                    : element.textContent

                const matches = isNumeric
                    ? textContent === query
                    : textContent.toLowerCase().includes(query)

                if (matches) {
                    this.showContainer(container)
                    return
                }
            }
        }

        this.hideContainer(container)
    }

    /**
     * Shows the searchable container.
     */
    showContainer(container) {
        if (!container.classList.contains('search-highlight')) {
            container.classList.add('search-highlight')
        }
    }

    /**
     * Hides the searchable container.
     */
    hideContainer(container) {
        if (container.classList.contains('search-highlight')) {
            container.classList.remove('search-highlight')
        }
    }

    /**
     * Clears the search input and resets all searchable containers.
     */
    clear() {
        this.inputTarget.value = ""
        this.performSearch()
    }
})
