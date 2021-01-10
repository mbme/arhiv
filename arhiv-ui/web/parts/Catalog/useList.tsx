import * as React from 'react'
import { noop, withoutUndefined } from '@v/utils'
import {
  API,
  IDocument,
  IDocumentFilter,
  IListPage,
  Matcher,
  OrderBy,
} from '../../api'

interface IList<D> {
  items?: D[]
  error?: any
  hasMore: boolean
  loadMore(): void
}

interface IOptions {
  matchers: Array<Matcher | undefined>
  pageSize?: number
  order: OrderBy[]
}

export function useList<D extends IDocument>(getOptions: () => IOptions, args: any[] = []): IList<D> {
  const [documentFilter, setDocumentFilter] = React.useState<IDocumentFilter>()

  const [items, setItems] = React.useState<D[]>()
  const [hasMore, setHasMore] = React.useState(false)
  const [error, setError] = React.useState<any>()

  React.useEffect(() => {
    setItems(undefined)
    setHasMore(false)
    setError(undefined)

    const {
      matchers,
      pageSize,
      order,
    } = getOptions()

    setDocumentFilter({
      pageOffset: 0,
      pageSize,
      matchers: withoutUndefined(matchers),
      order,
    })
  }, args)

  React.useEffect(() => {
    if (!documentFilter) {
      return noop
    }

    let relevant = true

    API.list<D>(documentFilter).then((page: IListPage<D>) => {
      if (relevant) {
        setItems(currentItems => [...(currentItems || []), ...page.items])
        setHasMore(page.hasMore)
      }
    }).catch(setError)

    return () => {
      relevant = false
    }
  }, [documentFilter])

  return {
    items,
    hasMore,
    error,

    loadMore() {
      setDocumentFilter((currentFilter) => {
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
    }
  }
}
