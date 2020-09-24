import { createContext } from '@v/web-utils'
import { Procedure } from '@v/utils'
import { Cell } from '@v/reactive'

export type FocusManagerMode = 'row' | 'column'

export class FocusManager {
  private _nodes = new Set<HTMLElement>()

  readonly selected$ = new Cell<HTMLElement | null>(null)
  readonly active$ = new Cell<boolean>(false)

  constructor(
    private _mode: FocusManagerMode,
  ) {}

  registerNode(node: HTMLElement): Procedure {
    this._nodes.add(node)

    const onMouseEnter = () => {
      this.selected$.value = node
    }

    node.addEventListener('mouseenter', onMouseEnter)

    return () => {
      node.removeEventListener('mouseenter', onMouseEnter)
      this._nodes.delete(node)
    }
  }

  activate() {
    if (this.active$.value) {
      return
    }

    this.active$.value = true
    if (this.selected$.value) {
      return
    }

    let firstEl = null
    for (const node of this._nodes) {
      if (!firstEl) {
        firstEl = node
        continue
      }

      if (this._getOffset(node) < this._getOffset(firstEl)) {
        firstEl = node
      }
    }

    if (firstEl) {
      this.selected$.value = firstEl
    }
  }

  deactivate() {
    this.active$.value = false
  }

  private _getOffset(node: HTMLElement): number {
    const rect = node.getBoundingClientRect()

    if (this._mode === 'row') {
      return rect.x + window.scrollX
    }

    return rect.y + window.scrollY
  }

  selectPrevious() {
    if (!this.active$.value || !this.selected$.value) {
      return
    }

    const currentOffset = this._getOffset(this.selected$.value)

    let prevEl = null
    for (const node of this._nodes) {
      const offset = this._getOffset(node)

      if (offset >= currentOffset) {
        continue
      }

      if (!prevEl || offset > this._getOffset(prevEl)) {
        prevEl = node
      }
    }

    if (prevEl) {
      this.selected$.value = prevEl
    }
  }

  selectNext() {
    if (!this.active$.value || !this.selected$.value) {
      return
    }

    const currentOffset = this._getOffset(this.selected$.value)

    let nextEl = null
    for (const node of this._nodes) {
      const offset = this._getOffset(node)

      if (offset <= currentOffset) {
        continue
      }

      if (!nextEl || offset < this._getOffset(nextEl)) {
        nextEl = node
      }
    }

    if (nextEl) {
      this.selected$.value = nextEl
    }
  }

  activateSelected() {
    if (!this.active$.value) {
      return
    }

    this.selected$.value?.click()
  }
}

export const FocusManagerContext = createContext<FocusManager>()
