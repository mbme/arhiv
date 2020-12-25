import * as React from 'react'
import {
  ProgressLocker,
  useForm,
  Input,
  Box,
  Button,
  Heading,
} from '@v/web-platform'
import { useDebounced } from '@v/web-utils'
import { ErrorBlock } from '../ErrorBlock'
import { useDataDescription } from '../../data-manager'
import { IDocument, Matcher } from '../../api'
import { useList } from './useList'
import { CatalogEntry } from './CatalogEntry'
import { partitionBy } from '@v/utils'

function createRule(field?: string) {
  if (!field) {
    return () => false
  }

  let lastVal: any = undefined
  return (item: IDocument): boolean => {
    const currentVal = item.data[field]
    const result = currentVal !== lastVal
    lastVal = currentVal

    return result
  }
}

interface IProps {
  documentType: string
  collectionMatcher?: Matcher
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
  } = useList({
    matchers: [
      { Type: { documentType } },
      collectionMatcher,
      debouncedFilter ? { FuzzyField: { selector: `$.${titleField}`, pattern: debouncedFilter } } : undefined,
    ],
    pageSize: uiOptions.catalog.pageSize,
    order: uiOptions.catalog.order,
  })

  if (error) {
    return (
      <ErrorBlock error={error} />
    )
  }

  const content = items ? (
    partitionBy(createRule(uiOptions.catalog.groupByField), items)
      .map(group => (
        <Box mb="large">
          <Heading>{group[0].data[uiOptions.catalog.groupByField!]} ({group.length})</Heading>
          {group.map(item => (
            <CatalogEntry
              key={item.id}
              document={item}
              showModificationDate={uiOptions.catalogEntry.showModificationDate}
              showDataFields={uiOptions.catalogEntry.showDataFields}
            />
          ))}
        </Box>
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
