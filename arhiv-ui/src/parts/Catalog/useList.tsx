import * as React from 'react'
import { noop, withoutUndefined } from '@v/utils'
import {
  API,
  IDocumentExt,
  IFilter,
  IListPage,
  Matcher,
  OrderBy,
} from '@v/arhiv-api'

interface IList<D> {
  items: D[]
  loading: boolean
  error?: any
  hasMore: boolean
  loadMore(): void
}

interface IState {
  items: IDocumentExt[]
  loading: boolean
  appendItems: boolean
  hasMore: boolean
  error?: any
}

interface IOptions {
  matchers: Array<Matcher | undefined>
  pageSize?: number
  order: OrderBy[]
}

export function useList(getOptions: () => IOptions, deps: any[] = []): IList<IDocumentExt> {
  const [filter, setFilter] = React.useState<IFilter>()

  const [state, setState] = React.useState<IState>({
    items: [],
    loading: false,
    appendItems: true,
    hasMore: false,
    error: undefined,
  })

  const updateState = (update: Partial<IState>) => setState(currentState => ({ ...currentState, ...update }))

  React.useEffect(() => {
    const {
      matchers,
      pageSize,
      order,
    } = getOptions()

    updateState({
      appendItems: false,
      error: undefined,
    })

    setFilter({
      pageOffset: 0,
      pageSize,
      matchers: withoutUndefined(matchers),
      order,
    })
  }, deps)

  React.useEffect(() => {
    if (!filter) {
      return noop
    }

    let relevant = true

    updateState({ loading: true })

    API.list(filter).then((page: IListPage<IDocumentExt>) => {
      if (!relevant) {
        return
      }

      updateState({
        items: state.appendItems ? [...state.items, ...page.items] : page.items,
        hasMore: page.hasMore,

        loading: false,
      })
    }).catch((e) => {
      if (!relevant) {
        return
      }

      updateState({
        items: [],
        hasMore: false,
        error: e,

        loading: false,
      })
    })

    return () => {
      relevant = false

      updateState({ loading: false })
    }
  }, [filter])

  return {
    items: state.items,
    hasMore: state.hasMore,
    error: state.error,
    loading: state.loading,

    loadMore() {
      setFilter((currentFilter) => {
        if (!currentFilter) {
          return currentFilter
        }

        if (!currentFilter.pageSize) {
          return currentFilter
        }

        return {
          ...currentFilter,
          pageOffset: (currentFilter.pageOffset || 0) + currentFilter.pageSize,
        }
      })

      updateState({ appendItems: true })
    }
  }
}
