import { Procedure, removeMut } from '@v/utils'
import { Cell, Observable } from '@v/reactive'
import { createLogger, Logger } from '@v/logger'

export type FocusManagerMode = 'row' | 'column'

export class FocusManager {
  private _nodes: HTMLElement[] = []
  private _log: Logger

  readonly enabled$ = new Cell(false, true)
  readonly selectedNode$ = new Cell<HTMLElement | null>(null, true)

  constructor(
    public readonly mode: FocusManagerMode,
    public readonly name: string,
  ) {
    this._log = createLogger(`FocusManager ${name}`)
  }

  registerNode(node: HTMLElement): Procedure {
    if (this._nodes.includes(node)) {
      throw new Error('node has been registered')
    }

    this._nodes.push(node)

    const onPointerMove = () => {
      if (this.enabled$.value) {
        this.selectedNode$.value = node
      }
    }

    node.addEventListener('pointermove', onPointerMove, { passive: true })

    return () => {
      node.removeEventListener('pointermove', onPointerMove)

      removeMut(this._nodes, node)
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

  private _getOffset(node: HTMLElement): [number, number] {
    const rect = node.getBoundingClientRect()

    const offsetX = rect.x + window.scrollX
    const offsetY = rect.y + window.scrollY

    if (this.mode === 'row') {
      return [offsetX, offsetY]
    }

    return [offsetY, offsetX]
  }

  private _sort() {
    this._nodes.sort((a: HTMLElement, b: HTMLElement) => {
      const offsetA = this._getOffset(a)
      const offsetB = this._getOffset(b)

      const diff = offsetA[0] - offsetB[0]

      // if difference of the main coordinate is big, use it
      if (Math.abs(diff) > 4) {
        return diff
      }

      // else compare secondary coordinate
      return offsetA[1] - offsetB[1]
    })
  }

  selectPreviousNode() {
    if (!this.enabled$.value || !this.selectedNode$.value) {
      return
    }
    this._log.debug('select previous node')

    this._sort()

    const currentPos = this._nodes.indexOf(this.selectedNode$.value)
    const prevPos = Math.max(0, currentPos - 1)

    this.selectedNode$.value = this._nodes[prevPos]
  }

  selectNextNode() {
    if (!this.enabled$.value || !this.selectedNode$.value) {
      return
    }
    this._log.debug('select next node')

    this._sort()

    const currentPos = this._nodes.indexOf(this.selectedNode$.value)
    const nextPos = Math.min(this._nodes.length - 1, currentPos + 1)

    this.selectedNode$.value = this._nodes[nextPos]
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
