import { Procedure, removeMut } from '@v/utils'
import { Cell, Observable } from '@v/reactive'
import { createLogger, Logger } from '@v/logger'

export type FocusManagerMode = 'row' | 'column'
type FocusChild = HTMLElement | FocusManager

export class FocusManager {
  private _children: FocusChild[] = []
  private _log: Logger
  private _destroyed = false

  readonly enabled$ = new Cell(false, true)
  readonly selectedChild$ = new Cell<FocusChild | undefined>(undefined, true)

  constructor(
    private _domNode: HTMLElement,
    public readonly mode: FocusManagerMode,
    public readonly name: string,
  ) {
    this._log = createLogger(`FocusManager ${name}`)
  }

  registerChild(child: FocusChild, autoFocus?: boolean): Procedure {
    if (this._destroyed) {
      throw new Error('FocusManager already destroyed')
    }

    if (this._children.includes(child)) {
      throw new Error('child has been registered')
    }

    if (child instanceof FocusManager) {
      this._log.debug(`add ${child.name}`)
    }

    this._children.push(child)

    // automatically select first added child if enabled
    const focusFirstChild = this.isEnabled() && this._children.length === 1

    if (autoFocus || focusFirstChild) {
      this._setSelected(child, autoFocus)
    }

    const onPointerMove = () => {
      if (this.isEnabled()) {
        this._setSelected(child)
      }
    }

    this._getNode(child).addEventListener('pointermove', onPointerMove, { passive: true })

    return () => {
      this._getNode(child).removeEventListener('pointermove', onPointerMove)

      if (this._destroyed) {
        return
      }

      const pos = this._children.indexOf(child)

      removeMut(this._children, child)

      if (this.isEnabled() && this.selectedChild$.value === child) {
        this._log.debug('unregisterChild: auto selecting first child')
        this.selectedChild$.value = undefined
        this._selectNearestChild(pos)
      }
    }
  }

  isEnabled(): boolean {
    return this.enabled$.value
  }

  enable() {
    if (this.isEnabled()) {
      return
    }

    this.enabled$.value = true

    this._log.debug('enabled')

    const child = this.selectedChild$.value

    if (!child) {
      this._log.debug('enable: auto select first child')
      this._selectNearestChild(0)
      return
    }

    // enable selected focus manager
    if (child instanceof FocusManager) {
      child.enable()
    }
  }

  disable() {
    if (!this.isEnabled()) {
      return
    }

    this.enabled$.value = false

    // disable children focus managers
    for (const child of this._children) {
      if (child instanceof FocusManager) {
        child.disable()
      }
    }

    this._log.debug('disabled')
  }

  destroy() {
    this._destroyed = true
    this._children.length = 0
  }

  selectPreviousChild() {
    if (!this.isEnabled() || !this._children.length) {
      return
    }
    this._log.debug('select previous child')

    this._sort()

    let prevPos = this._children.length - 1
    if (this.selectedChild$.value) {
      const currentPos = this._children.indexOf(this.selectedChild$.value)
      prevPos = Math.max(0, currentPos - 1)
    }

    this._setSelected(this._children[prevPos], true)
  }

  selectNextChild() {
    if (!this.isEnabled() || !this._children.length) {
      return
    }
    this._log.debug('select next child')

    this._sort()

    let nextPos = 0
    if (this.selectedChild$.value) {
      const currentPos = this._children.indexOf(this.selectedChild$.value)
      nextPos = Math.min(this._children.length - 1, currentPos + 1)
    }

    this._setSelected(this._children[nextPos], true)
  }

  activateSelectedChild() {
    if (!this.isEnabled() || !this.selectedChild$.value) {
      return
    }

    this._log.debug('activate selected')

    const child = this.selectedChild$.value

    if (child instanceof HTMLElement) {
      child.dispatchEvent(new Event('activate'))
    }
  }

  isChildSelected$(child: HTMLElement | undefined): Observable<boolean> {
    return this.enabled$.value$.switchMap((enabled) => {
      if (!enabled) {
        return Observable.from(false)
      }

      return this.selectedChild$.value$.map(value => !!value && value === child)
    })
  }

  isChildSelected(child: HTMLElement | undefined): boolean {
    const value = this.selectedChild$.value

    return !!value && value === child && this.isEnabled()
  }

  private _getNode(child: FocusChild): HTMLElement {
    if (child instanceof FocusManager) {
      return child._domNode
    }

    return child
  }

  private _getOffset(child: FocusChild): [number, number] {
    const rect = this._getNode(child).getBoundingClientRect()

    const offsetX = rect.x + window.scrollX
    const offsetY = rect.y + window.scrollY

    if (this.mode === 'row') {
      return [offsetX, offsetY]
    }

    return [offsetY, offsetX]
  }

  private _sort() {
    this._children.sort((a: FocusChild, b: FocusChild) => {
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

  private _setSelected(child: FocusChild, scrollIntoView = false) {
    const prevChild = this.selectedChild$.value

    if (child === prevChild) {
      // make sure this focus manager is enabled
      if (child instanceof FocusManager) {
        child.enable()
      }

      return
    }

    if (prevChild instanceof FocusManager) {
      prevChild.disable()
    }

    this.selectedChild$.value = child

    if (child instanceof FocusManager) {
      child.enable()
    }

    if (scrollIntoView) {
      this._getNode(child).scrollIntoView({
        behavior: 'auto',
        block: 'center',
      })
    }
  }

  private _selectNearestChild(pos: number) {
    if (!this._children.length) {
      return
    }

    this._sort()

    const nextPos = Math.min(pos, this._children.length - 1)

    this._log.debug(`select child ${nextPos}`)
    this._setSelected(this._children[nextPos])
  }
}
