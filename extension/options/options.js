class Editor {
  constructor(command, gui) {
    this.command = command
    this.gui = gui
  }
}
const editors = {
  'nvim': new Editor('nvim', false),
  'vim': new Editor('vim', false),
  'emacs': new Editor('emacs', true),
  'kak': new Editor('kak', false),
  'neovide': new Editor('neovide --nofork', true),
  'gvim': new Editor('gvim --nofork', true),
}
const homebrewDefaultDir = '/usr/local/bin/'

const editorSelect = document.getElementById('editor')
const terminalRow = document.getElementById('terminal-row')
const terminalSelect = document.getElementById('terminal')
const shellRow = document.getElementById('shell-row')
const shellInput = document.getElementById('shell')
const templateInput = document.getElementById('template')
const upstreamTemplateRow = document.getElementById('upstream-template-row')
const upstreamTemplateInput = document.getElementById('upstream-template')
const upstreamTemplateSyncButton = document.getElementById('upstream-template-sync')
const bypassVersionCheckInput = document.getElementById('bypass-version-check')
const applyButton = document.getElementById('apply')

async function updateOptionsForEditor(editor) {
  // don't touch any values except for upstream template here, since this
  // function also gets called from loadSettings() and we want to show users
  // their current settings
  if (editor === 'custom') {
    showElement(shellRow)
    templateInput.removeAttribute('disabled')
    hideElement(terminalRow)
  } else {
    hideElement(shellRow)
    templateInput.setAttribute('disabled', 'true')
    const editorConfig = editors[editor]
    if (editorConfig.gui) {
      hideElement(terminalRow)
    } else {
      showElement(terminalRow)
    }
    await updateUpstreamTemplate()
    if (templateInput.value !== upstreamTemplateInput.value) {
      showElement(upstreamTemplateRow)
    } else {
      hideElement(upstreamTemplateRow)
    }
  }
}

editorSelect.onchange = async (e) => {
  const editor = e.target.value
  await updateTemplate()
  await updateOptionsForEditor(editor)
}
terminalSelect.onchange = async () => {
  const editor = editorSelect.value
  await updateTemplate()
  await updateOptionsForEditor(editor)
}

upstreamTemplateSyncButton.onclick = async () => {
  templateInput.value = upstreamTemplateInput.value
  hideElement(upstreamTemplateRow)
}

let applyButtonCountdown = null
applyButton.onclick = async () => {
  if (applyButtonCountdown !== null) {
    clearInterval(applyButtonCountdown)
  }
  applyButtonCountdown = setInterval(() => {
    applyButton.setAttribute('value', 'Apply')
    applyButton.removeAttribute('disabled')
  }, 750)
  applyButton.setAttribute('value', 'Saved!')
  applyButton.setAttribute('disabled', 'true')
  await saveSettings()
}

function hideElement(element) {
  element.style = 'display: none;'
}

function showElement(element) {
  element.style = ''
}

const templateTempFileName = '"/path/to/temp.eml"'
async function generateTemplate() {
  const editor = editorSelect.value
  if (editor === 'custom') {
    return null
  }

  const platform = await browser.runtime.getPlatformInfo()
  const editorConfig = editors[editor]
  const editorCommand = platform.os === browser.runtime.PlatformOs.MAC ? homebrewDefaultDir + editorConfig.command : editorConfig.command
  if (editorConfig.gui) {
    return editorCommand + " " + templateTempFileName
  }

  let terminalCommand = platform.os === browser.runtime.PlatformOs.MAC ? homebrewDefaultDir : ''
  switch (terminalSelect.value) {
    case 'kitty':
      terminalCommand += 'kitty --start-as=normal --override=macos_quit_when_last_window_closed=yes --'
      break
    case 'alacritty':
      terminalCommand += 'alacritty -e'
      break
    case 'konsole':
      terminalCommand += 'konsole -e'
      break
  }
  return terminalCommand + " " + editorCommand + " " + templateTempFileName
}
async function updateTemplate() {
  templateInput.value = await generateTemplate()
}
async function updateUpstreamTemplate() {
  upstreamTemplateInput.value = await generateTemplate()
}

async function saveSettings() {
  const editor = editorSelect.value
  const terminal = terminalSelect.value
  const shell = editor === 'custom' ? shellInput.value : 'sh'
  const template = templateInput.value
  const bypassVersionCheck = bypassVersionCheckInput.checked
  await browser.storage.local.set({
    editor: editor,
    terminal: terminal,
    shell: shell,
    template: template,
    bypassVersionCheck: bypassVersionCheck,
  })
}

async function loadSettings() {
  const settings = await browser.storage.local.get(['editor', 'terminal', 'shell', 'template', 'bypassVersionCheck'])
  if (settings.editor) {
    editorSelect.value = settings.editor
    terminalSelect.value = settings.terminal
    shellInput.value = settings.shell
    templateInput.value = settings.template
    bypassVersionCheckInput.checked = settings.bypassVersionCheck
    await updateOptionsForEditor(settings.editor)
  } else {
    await updateTemplate()
  }
}

loadSettings()
