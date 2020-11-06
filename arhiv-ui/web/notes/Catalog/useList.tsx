import * as React from 'react'
import { noop } from '@v/utils'
import { API, IDocumentFilter, IListPage, Note } from '../../api'

const PAGE_SIZE = 10

export function useList(pattern: string) {
  const [documentFilter, setDocumentFilter] = React.useState<IDocumentFilter<'note'>>()

  const [items, setItems] = React.useState<Note[]>()
  const [hasMore, setHasMore] = React.useState(false)
  const [error, setError] = React.useState<any>()

  React.useEffect(() => {
    setItems(undefined)
    setHasMore(false)
    setError(undefined)

    setDocumentFilter({
      type: 'note',
      pageOffset: 0,
      pageSize: PAGE_SIZE,
      skipArchived: true,
      matcher: pattern ? {
        selector: '$.name',
        pattern,
      } : undefined,
    })
  }, [pattern])

  React.useEffect(() => {
    if (!documentFilter) {
      return noop
    }

    let relevant = true

    API.list<Note>(documentFilter).then((page: IListPage<Note>) => {
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
