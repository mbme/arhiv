import * as React from 'react'
import { Store, createContext } from '~/web-utils'
import { Counter, Dict, Procedure, replaceAtMut } from '~/utils'
import { Document, ArhivReplica } from '~/arhiv/replica'
import { createDocument } from './document-types'

interface IState {
  showCatalog: boolean
  filter: string
  items: Document[]
  focused?: Document
}

export class WorkspaceStore extends Store<IState> {
  private _listeners: Dict<Procedure> = {}

  constructor(
    private _arhiv: ArhivReplica,
  ) {
    super({
      filter: '',
      items: [],
      showCatalog: true,
      focused: undefined,
    })
  }

  updateFilter(filter: string) {
    this._setState({
      ...this.state,
      showCatalog: filter.length ? true : this.state.showCatalog,
      filter,
    })
  }

  isDocumentOpen(id: string): boolean {
    return !!this._listeners[id]
  }

  openDocument(id: string) {
    if (this.isDocumentOpen(id)) {
      return
    }

    this._listeners[id] = this._arhiv.documents.getDocument$(id).subscribe({
      next: (document) => {
        const items = [...this.state.items]
        const i = items.findIndex(item => item.id === id)

        if (i === -1) {
          items.unshift(document)

          this._setState({
            ...this.state,
            showCatalog: false,
            focused: document,
            items,
          })
          return
        }

        replaceAtMut(items, i, document)

        this._setState({
          ...this.state,
          items,
        })
      },
    })
  }

  closeDocument(id: string) {
    const unsub = this._listeners[id]
    if (!unsub) {
      return
    }

    unsub()
    delete this._listeners[id]

    this._setState({
      ...this.state,
      items: this.state.items.filter(item => item.id !== id),
    })
  }

  async createDocument(type: string) {
    const document = await createDocument(type, this._arhiv)

    this._setState({
      ...this.state,
      showCatalog: false,
      focused: document,
      items: [document, ...this.state.items],
    })
  }

  showCatalog(show: boolean) {
    this._setState({
      ...this.state,
      showCatalog: show,
    })
  }
}

export const WorkspaceStoreContext = createContext<WorkspaceStore>()

const _counter = new Counter()
export function useWorkspaceStore() {
  const store = WorkspaceStoreContext.use()

  const [, setCounter] = React.useState(_counter.incAndGet())

  React.useEffect(() => store.state$.subscribe({
    next() {
      setCounter(_counter.incAndGet())
    },
  }), [store])

  return store
}
