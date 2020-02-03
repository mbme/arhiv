import * as React from 'react'
import { Store, createContext } from '~/web-utils'
import { Counter } from '~/utils'

interface IDocumentItem {
  _type: 'document'
  id: string
}
interface INewDocumentItem {
  _type: 'new-document'
  type: string
  tempId: number
}
export type WorkspaceItem = IDocumentItem | INewDocumentItem

interface IState {
  filter: string
  items: WorkspaceItem[]
  showCatalog: boolean
  focused?: WorkspaceItem
}

export class WorkspaceStore extends Store<IState> {
  constructor() {
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
    return !!this.state.items.find(item => item._type === 'document' && item.id === id)
  }

  openDocument(id: string) {
    if (this.isDocumentOpen(id)) {
      return
    }

    const newItem: IDocumentItem = {
      _type: 'document',
      id,
    }

    this._setState({
      ...this.state,
      showCatalog: false,
      focused: newItem,
      items: [newItem, ...this.state.items],
    })
  }

  closeDocument(id: string) {
    if (!this.isDocumentOpen(id)) {
      return
    }

    this._setState({
      ...this.state,
      items: this.state.items.filter(item => item._type !== 'document' || item.id !== id),
    })
  }

  createDocument(type: string) {
    const newItem: INewDocumentItem = {
      _type: 'new-document',
      type,
      tempId: 1,
    }

    this._setState({
      ...this.state,
      showCatalog: false,
      focused: newItem,
      items: [newItem, ...this.state.items],
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
