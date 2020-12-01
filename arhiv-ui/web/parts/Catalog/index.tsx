import * as React from 'react'
import {
  ProgressLocker,
  useForm,
  Input,
  Box,
  Button,
} from '@v/web-platform'
import { useDebounced } from '@v/web-utils'
import { CatalogEntry } from './CatalogEntry'
import { ErrorBlock, Frame, Action } from '../../parts'
import { useList } from './useList'
import { IDocument, IMatcher } from '../../api'
import { useDataDescription } from '../../data-manager'

interface IProps {
  documentType: string
  title: string
  getMatchers(filter: string): IMatcher[]
  onAdd(): void
  onActivate(document: IDocument): void
}

export function Catalog(props: IProps) {
  const {
    title,
    documentType,
    getMatchers,
    onAdd,
    onActivate,
  } = props

  const {
    Form,
    values: {
      filter = '',
    },
  } = useForm()

  const dataDescription = useDataDescription(documentType)
  const debouncedFilter = useDebounced(filter, 300)
  const matchers = React.useMemo(() => getMatchers(debouncedFilter), [debouncedFilter])
  const {
    items,
    hasMore,
    error,
    loadMore,
  } = useList(matchers)

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
          dataDescription={dataDescription}
          onActivate={onActivate}
        />
      ))
  ) : (
    <ProgressLocker />
  )

  const actions = (
    <Action
      type="action"
      onClick={onAdd}
    >
      Add
    </Action>
  )

  return (
    <Frame
      actions={actions}
      title={title}
    >
      <Box
        pb="small"
        width="100%"
      >
        <Form>
          <Input
            label=""
            name="filter"
            placeholder="Filter documents"
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
    </Frame>
  )
}
