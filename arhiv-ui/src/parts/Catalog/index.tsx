import * as React from 'react'
import {
  ProgressLocker,
  useForm,
  Input,
  Box,
  Button,
} from '@v/web-platform'
import { useDebounced } from '@v/web-utils'
import { ErrorBlock } from '../ErrorBlock'
import { Matcher } from '../../api'
import { useList } from './useList'
import { CatalogEntries } from './CatalogEntries'
import { getUIOptions, CatalogOptionsOverrides } from './options'

export { CatalogOptionsOverrides }

interface IProps {
  documentType: string
  collectionMatcher?: Matcher
  options?: CatalogOptionsOverrides
}

export function Catalog({ documentType, collectionMatcher, options }: IProps) {
  const {
    Form,
    values: {
      filter = '',
    },
  } = useForm()

  const uiOptions = React.useMemo(() => getUIOptions(options), [options])

  const debouncedFilter = useDebounced(filter, 300)

  const {
    items,
    hasMore,
    error,
    loadMore,
  } = useList(() => ({
    matchers: [
      { Type: { documentType } },
      collectionMatcher,
      debouncedFilter ? { FuzzyField: { selector: '$', pattern: debouncedFilter } } : undefined,
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
        pr="medium"
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
