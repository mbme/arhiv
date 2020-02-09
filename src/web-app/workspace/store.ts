import * as React from 'react'
import { Store, createContext } from '~/web-utils'
import { Counter, Dict, Procedure, replaceAtMut } from '~/utils'
import { Document, ArhivReplica } from '~/arhiv/replica'
import { IDocumentModule } from './document-types/types'

interface IState {
  showCatalog: boolean
  filter: string
  items: Document[]
  focusedId?: string
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
      focusedId: undefined,
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

  private _watchDocument(id: string) {
    const unsub = this._arhiv.documents.getDocument$(id).subscribe({
      next: (document) => {
        const items = [...this.state.items]
        const i = items.findIndex(item => item.id === id)

        if (i === -1) {
          items.unshift(document)

          this._setState({
            ...this.state,
            showCatalog: false,
            focusedId: document.id,
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

    this._listeners[id] = unsub
  }

  openDocument(id: string) {
    if (this.isDocumentOpen(id)) {
      this._setState({ // just show the document
        ...this.state,
        showCatalog: false,
        focusedId: id,
      })
      return
    }

    this._watchDocument(id)
  }

  closeDocument(id: string) {
    const unsub = this._listeners[id]
    if (!unsub) {
      return
    }

    unsub()
    delete this._listeners[id]

    const items = this.state.items.filter(item => item.id !== id)
    this._setState({
      ...this.state,
      items,
      showCatalog: items.length ? this.state.showCatalog : true,
    })
  }

  async createDocument(module: IDocumentModule) {
    const document = await this._arhiv.documents.create(module.type, module.initialProps)

    const unsub = document.isNew$()
      .filter(isNew => !isNew)
      .take(1)
      .subscribe({
        next: () => {
          unsub()
          this._watchDocument(document.id)
        },
      })
    this._listeners[document.id] = unsub

    this._setState({
      ...this.state,
      showCatalog: false,
      focusedId: document.id,
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
