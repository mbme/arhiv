import * as React from 'react'
import { noop, Obj } from '@v/utils'
import { API, IDocument, IDocumentFilter, IListPage } from '../../api'

const PAGE_SIZE = 10

interface IList<D> {
  items?: D[]
  error?: any
  hasMore: boolean
  loadMore(): void
}

export function useList<D extends IDocument<T, P>, T extends string = string, P extends Obj = Obj>(
  type: T,
  selector: string,
  pattern: string,
): IList<D> {
  const [documentFilter, setDocumentFilter] = React.useState<IDocumentFilter<typeof type>>()

  const [items, setItems] = React.useState<D[]>()
  const [hasMore, setHasMore] = React.useState(false)
  const [error, setError] = React.useState<any>()

  React.useEffect(() => {
    setItems(undefined)
    setHasMore(false)
    setError(undefined)

    setDocumentFilter({
      type,
      pageOffset: 0,
      pageSize: PAGE_SIZE,
      skipArchived: true,
      matcher: pattern ? {
        selector,
        pattern,
      } : undefined,
    })
  }, [pattern])

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

        return {
          ...currentFilter,
          pageOffset: (currentFilter.pageOffset || 0) + PAGE_SIZE,
        }
      })
    }
  }
}
