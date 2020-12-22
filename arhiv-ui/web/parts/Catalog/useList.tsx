import * as React from 'react'
import { noop, withoutUndefined } from '@v/utils'
import {
  API,
  IDocument,
  IDocumentFilter,
  IListPage,
  IMatcher,
  OrderBy,
} from '../../api'

interface IList<D> {
  items?: D[]
  error?: any
  hasMore: boolean
  loadMore(): void
}

interface IOptions {
  matchers: Array<IMatcher | undefined>
  pageSize?: number
  order: OrderBy[]
}

export function useList<D extends IDocument>({ matchers, pageSize, order }: IOptions): IList<D> {
  const [documentFilter, setDocumentFilter] = React.useState<IDocumentFilter>()

  const [items, setItems] = React.useState<D[]>()
  const [hasMore, setHasMore] = React.useState(false)
  const [error, setError] = React.useState<any>()

  React.useEffect(() => {
    setItems(undefined)
    setHasMore(false)
    setError(undefined)

    setDocumentFilter({
      pageOffset: 0,
      pageSize,
      matchers: withoutUndefined(matchers),
      order,
    })
  }, [JSON.stringify(matchers), pageSize]) // FIXME shallow equality

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
      if (!pageSize) {
        return
      }

      setDocumentFilter((currentFilter) => {
        if (!currentFilter) {
          return currentFilter
        }

        return {
          ...currentFilter,
          pageOffset: (currentFilter.pageOffset || 0) + pageSize,
        }
      })
    }
  }
}
