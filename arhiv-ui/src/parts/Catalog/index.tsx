import * as React from 'react'
import {
  ProgressLocker,
  useForm,
  Input,
  Box,
  Button,
} from '@v/web-platform'
import { useDebounced } from '@v/web-utils'
import { countSubstring } from '@v/utils'
import { ErrorBlock } from '../ErrorBlock'
import { Matcher } from '../../api'
import { useList } from './useList'
import { CatalogEntries } from './CatalogEntries'
import { getUIOptions, CatalogOptionsOverrides } from './options'

export { CatalogOptionsOverrides }

function isValidFilter(filter: string): boolean {
  return countSubstring(filter, '"') % 2 === 0
}

function normalizeFilter(filter: string): string {
  return filter
    .split(' ')
    .map(item => item.trim())
    .filter(item => item.length > 0)
    .map((item: string) => {
      if (item.endsWith('*')) {
        return item
      }

      return item + '*'
    })
    .join(' ')
}

interface IProps {
  documentType: string
  collectionMatcher?: Matcher
  options?: CatalogOptionsOverrides
}

export function Catalog({ documentType, collectionMatcher, options }: IProps) {
  const uiOptions = React.useMemo(() => getUIOptions(options), [options])

  const {
    Form,
    values: {
      filter = '',
    },
  } = useForm()

  const isValid = isValidFilter(filter)
  const debouncedFilter = useDebounced(normalizeFilter(filter), 600, isValid)

  const {
    items,
    hasMore,
    error,
    loadMore,
  } = useList(() => ({
    matchers: [
      { Type: { documentType } },
      collectionMatcher,
      debouncedFilter ? { Search: { pattern: debouncedFilter } } : undefined,
    ],
    pageSize: uiOptions.pageSize,
    order: uiOptions.order,
  }), [debouncedFilter])

  if (error) {
    return (
      <ErrorBlock error={error} />
    )
  }

  const content = items ? (
    <CatalogEntries
      items={items}
      uiOptions={uiOptions}
    />
  ) : (
    <ProgressLocker />
  )

  return (
    <>
      <Box
        pb="small"
        width="100%"
      >
        <Form>
          <Input
            label=""
            name="filter"
            placeholder={`Filter ${documentType}s`}
          />
        </Form>
      </Box>

      <Box
        flex="1 1 auto"
        overflowY="auto"
        width="100%"
      >
        {content}

        {hasMore && (
          <Button
            variant="link"
            onClick={loadMore}
          >
            Load more
          </Button>
        )}
      </Box>
    </>
  )
}
