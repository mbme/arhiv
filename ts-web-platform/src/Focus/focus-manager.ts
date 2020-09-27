import { Procedure } from '@v/utils'
import { Cell, Observable } from '@v/reactive'
import { createLogger, Logger } from '@v/logger'

export type FocusManagerMode = 'row' | 'column'

export class FocusManager {
  private _nodes = new Set<HTMLElement>()
  private _log: Logger

  readonly selected$ = new Cell<HTMLElement | null>(null)
  readonly active$ = new Cell(false) // FIXME rename to enabled$

  constructor(
    private _mode: FocusManagerMode,
    public readonly name: string,
  ) {
    this._log = createLogger(`FocusManager ${name}`)
  }

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

  activate() { // FIXME rename enable/disable
    if (this.active$.value) {
      return
    }

    this._log.debug('activated')

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
    if (!this.active$.value) {
      return
    }

    this._log.debug('deactivated')
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
    this._log.debug('select previous')

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
    this._log.debug('select next')

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

    this._log.debug('activate selected')

    this.selected$.value?.dispatchEvent(new Event('activate'))
  }

  isSelected$(node: HTMLElement | null): Observable<boolean> {
    return this.selected$.value$.map(value => !!value && value === node)
  }

  isSelected(node: HTMLElement | null): boolean {
    const value = this.selected$.value

    return !!value && value === node
  }

  scrollSelectedIntoView() { // FIXME improve this
    this.selected$.value?.scrollIntoView({
      behavior: 'smooth',
      block: 'nearest',
    })
  }
}
