export interface IKeybinding {
  code: string
  ctrlKey?: boolean
  altKey?: boolean
  shiftKey?: boolean
  metaKey?: boolean
  action(e: KeyboardEvent): void
}

export function isMatchingEvent(keybinding: IKeybinding, e: KeyboardEvent): boolean {
  return e.code === keybinding.code
      && e.ctrlKey === (keybinding.ctrlKey || false)
      && e.altKey === (keybinding.altKey || false)
      && e.shiftKey === (keybinding.shiftKey || false)
      && e.metaKey === (keybinding.metaKey || false)
}

export function createKeybindingsHandler(...keybindings: IKeybinding[]) {
  return function keybindingsHandler(e: KeyboardEvent) {
    const keybinding = keybindings.find(item => isMatchingEvent(item, e))

    if (keybinding) {
      keybinding.action(e)
    }
  }
}
