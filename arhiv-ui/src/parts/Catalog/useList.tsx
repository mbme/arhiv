import * as React from 'react'
import { noop, withoutUndefined } from '@v/utils'
import {
  API,
  IDocumentExt,
  IFilter,
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

export function useList(getOptions: () => IOptions, args: any[] = []): IList<IDocumentExt> {
  const [filter, setFilter] = React.useState<IFilter>()

  const [items, setItems] = React.useState<IDocumentExt[]>()
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

    setFilter({
      pageOffset: 0,
      pageSize,
      matchers: withoutUndefined(matchers),
      order,
    })
  }, args)

  React.useEffect(() => {
    if (!filter) {
      return noop
    }

    let relevant = true

    API.list(filter).then((page: IListPage<IDocumentExt>) => {
      if (relevant) {
        setItems(currentItems => [...(currentItems || []), ...page.items])
        setHasMore(page.hasMore)
      }
    }).catch(setError)

    return () => {
      relevant = false
    }
  }, [filter])

  return {
    items,
    hasMore,
    error,

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
    }
  }
}
