import {
  removeAtMut,
} from '@v/utils'

export interface IKeybinding {
  code: string
  ctrlKey?: boolean
  altKey?: boolean
  shiftKey?: boolean
  metaKey?: boolean
  preventDefault?: boolean
  action(e: KeyboardEvent): void
}

function isMatchingEvent(keybinding: IKeybinding, e: KeyboardEvent): boolean {
  return e.code === keybinding.code
      && e.ctrlKey === (keybinding.ctrlKey || false)
      && e.altKey === (keybinding.altKey || false)
      && e.shiftKey === (keybinding.shiftKey || false)
      && e.metaKey === (keybinding.metaKey || false)
}

const TEXT_EDITOR_TAGS = ['input', 'textarea', 'select']
function isTextEditorEvent(e: KeyboardEvent): boolean {
  const tagName = (e.target as HTMLElement)?.tagName.toLowerCase()

  return TEXT_EDITOR_TAGS.includes(tagName)
}

interface IKeybindings {
  keybindings: IKeybinding[]
  priority: number
}

export class HotkeysResolver {
  private _hotkeys: IKeybindings[] = []

  add(priority: number, keybindings: IKeybinding[]) {
    this._hotkeys.push({
      keybindings,
      priority,
    })

    this._sort()
  }

  remove(keybindings: IKeybinding[]): boolean {
    const pos = this._hotkeys.findIndex(item => item.keybindings === keybindings)

    removeAtMut(this._hotkeys, pos)

    this._sort()

    return pos > -1
  }

  private _sort() {
    this._hotkeys.sort((a, b) => b.priority - a.priority)
  }

  private _onKeyDown = (e: KeyboardEvent) => {
    if (isTextEditorEvent(e)) {
      return
    }

    for (const hotkeys of this._hotkeys) {
      const keybinding = hotkeys.keybindings.find(item => isMatchingEvent(item, e))

      if (!keybinding) {
        continue
      }

      if (keybinding.preventDefault) {
        e.preventDefault()
      }

      keybinding.action(e)

      return
    }
  }

  registerDocument(document: Document) {
    document.addEventListener('keydown', this._onKeyDown)

    return () => {
      document.removeEventListener('keydown', this._onKeyDown)
    }
  }
}
