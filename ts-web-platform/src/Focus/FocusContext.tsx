import { createContext } from '@v/web-utils'
import { Procedure } from '@v/utils'
import { Cell } from '@v/reactive'

export type FocusManagerMode = 'row' | 'column'

// FIXME navigation manager / keybindings
export class FocusManager {
  private _nodes = new Set<HTMLElement>()

  readonly selected$ = new Cell<HTMLElement | null>(null)
  private _regionNode?: HTMLElement

  constructor(
    private _mode: FocusManagerMode,
  ) {}

  registerNode(node: HTMLElement): Procedure {
    this._nodes.add(node)
    // FIXME on mouse enter

    return () => {
      this.unregisterNode(node)
    }
  }

  unregisterNode(node: HTMLElement) {
    this._nodes.delete(node)
  }

  registerRegionNode(node: HTMLElement) {
    // FIXME on mouse enter
    this._regionNode = node
  }

  unregisterRegionNode() {
    this._regionNode = undefined
  }

  private _getOffset(node: HTMLElement): number {
    const rect = node.getBoundingClientRect()

    if (this._mode === 'row') {
      return rect.x + window.scrollX
    }

    return rect.y + window.scrollY
  }

  selectPrevious() {
    if (!this._nodes || !this.selected$.value) {
      return
    }

    const currentOffset = this._getOffset(this.selected$.value)

    let prevEl = this.selected$.value
    for (const node of this._nodes) {
      const offset = this._getOffset(node)

      if (offset >= currentOffset) {
        continue
      }

      if (offset > this._getOffset(prevEl)) {
        prevEl = node
      }
    }

    this.selected$.value = prevEl
  }

  selectNext() {
    if (!this._nodes || !this.selected$.value) {
      return
    }

    const currentOffset = this._getOffset(this.selected$.value)

    let nextEl = this.selected$.value
    for (const node of this._nodes) {
      const offset = this._getOffset(node)

      if (offset <= currentOffset) {
        continue
      }

      if (offset < this._getOffset(nextEl)) {
        nextEl = node
      }
    }

    this.selected$.value = nextEl
  }

  activateSelected() {
    this.selected$.value?.click()
  }

  focusParent() {}
}

export const FocusManagerContext = createContext<FocusManager>()


// register
// unregister
// focused
// isFocusedRegion
