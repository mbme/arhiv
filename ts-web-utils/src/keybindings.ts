export interface IKeybinding {
  code: string
  ctrlKey?: boolean
  altKey?: boolean
  shiftKey?: boolean
  metaKey?: boolean
  action(e: KeyboardEvent): void
}

export function createKeybindingsHandler(...keybindings: IKeybinding[]) {
  return function keybindingsHandler(e: KeyboardEvent) {
    for (const keybinding of keybindings) {
      if (e.code === keybinding.code
        && e.ctrlKey === (keybinding.ctrlKey || false)
        && e.altKey === (keybinding.altKey || false)
        && e.shiftKey === (keybinding.shiftKey || false)
        && e.metaKey === (keybinding.metaKey || false)) {
        keybinding.action(e)

        return
      }
    }
  }
}
