import { Procedure } from '@v/utils'
import { Cell, Observable } from '@v/reactive'
import { createLogger, Logger } from '@v/logger'

export type FocusManagerMode = 'row' | 'column'

export class FocusManager {
  private _nodes = new Set<HTMLElement>()
  private _log: Logger

  readonly enabled$ = new Cell(false)
  readonly selectedNode$ = new Cell<HTMLElement | null>(null)

  constructor(
    private _mode: FocusManagerMode,
    public readonly name: string,
  ) {
    this._log = createLogger(`FocusManager ${name}`)
  }

  registerNode(node: HTMLElement): Procedure {
    this._nodes.add(node)

    const onMouseEnter = () => {
      this.selectedNode$.value = node
    }

    node.addEventListener('mouseenter', onMouseEnter)

    return () => {
      node.removeEventListener('mouseenter', onMouseEnter)
      this._nodes.delete(node)
    }
  }

  enable() {
    if (this.enabled$.value) {
      return
    }

    this._log.debug('enabled')

    this.enabled$.value = true
    if (this.selectedNode$.value) {
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
      this.selectedNode$.value = firstEl
    }
  }

  disable() {
    if (!this.enabled$.value) {
      return
    }

    this._log.debug('disabled')
    this.enabled$.value = false
  }

  private _getOffset(node: HTMLElement): number {
    const rect = node.getBoundingClientRect()

    if (this._mode === 'row') {
      return rect.x + window.scrollX
    }

    return rect.y + window.scrollY
  }

  selectPreviousNode() {
    if (!this.enabled$.value || !this.selectedNode$.value) {
      return
    }
    this._log.debug('select previous node')

    const currentOffset = this._getOffset(this.selectedNode$.value)

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
      this.selectedNode$.value = prevEl
    }
  }

  selectNextNode() {
    if (!this.enabled$.value || !this.selectedNode$.value) {
      return
    }
    this._log.debug('select next node')

    const currentOffset = this._getOffset(this.selectedNode$.value)

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
      this.selectedNode$.value = nextEl
    }
  }

  activateSelectedNode() {
    if (!this.enabled$.value) {
      return
    }

    this._log.debug('activate selected')

    this.selectedNode$.value?.dispatchEvent(new Event('activate'))
  }

  isNodeSelected$(node: HTMLElement | null): Observable<boolean> {
    return this.selectedNode$.value$.map(value => !!value && value === node)
  }

  isNodeSelected(node: HTMLElement | null): boolean {
    const value = this.selectedNode$.value

    return !!value && value === node
  }

  scrollSelectedNodeIntoView() { // FIXME improve this
    this.selectedNode$.value?.scrollIntoView({
      behavior: 'smooth',
      block: 'nearest',
    })
  }
}
