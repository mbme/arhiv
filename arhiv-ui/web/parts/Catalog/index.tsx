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
import { useDataDescription } from '../../data-manager'
import { IMatcher } from '../../api'
import { useList } from './useList'
import { CatalogEntry } from './CatalogEntry'

interface IProps {
  documentType: string
  collectionMatcher?: IMatcher
}

export function Catalog({ documentType, collectionMatcher }: IProps) {
  const {
    Form,
    values: {
      filter = '',
    },
  } = useForm()

  const {
    titleField,
    uiOptions,
  } = useDataDescription(documentType)
  const debouncedFilter = useDebounced(filter, 300)

  const {
    items,
    hasMore,
    error,
    loadMore,
  } = useList([
    { selector: '$.type', pattern: documentType, fuzzy: false },
    collectionMatcher,
    debouncedFilter ? { selector: `$.${titleField}`, pattern: debouncedFilter, fuzzy: true } : undefined,
  ], uiOptions.catalog.pageSize)

  if (error) {
    return (
      <ErrorBlock error={error} />
    )
  }

  const content = items ? (
    items
      .map(item => (
        <CatalogEntry
          key={item.id}
          document={item}
          showModificationDate={uiOptions.catalogEntry.showModificationDate}
          showDataFields={uiOptions.catalogEntry.showDataFields}
        />
      ))
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
