import { removeMut } from '@v/utils'

export interface IKeybinding {
  code: string
  ctrlKey?: boolean
  altKey?: boolean
  shiftKey?: boolean
  metaKey?: boolean
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

export class HotkeysResolver {
  private _hotkeys: Array<IKeybinding[]> = []

  add(hotkeys: IKeybinding[]) {
    this._hotkeys.push(hotkeys)
  }

  remove(hotkeys: IKeybinding[]) {
    removeMut(this._hotkeys, hotkeys)
  }

  private _onKeyDown = (e: KeyboardEvent) => {
    if (isTextEditorEvent(e)) {
      return
    }

    for (let i = this._hotkeys.length - 1; i >= 0; i -= 1) {
      const hotkeys = this._hotkeys[i]

      const keybinding = hotkeys.find(item => isMatchingEvent(item, e))

      if (keybinding) {
        keybinding.action(e)

        return
      }
    }
  }

  registerDocument(document: Document) {
    document.addEventListener('keydown', this._onKeyDown)

    return () => {
      document.removeEventListener('keydown', this._onKeyDown)
    }
  }
}
