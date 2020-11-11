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
import { IDocument, Matcher } from '../../api'
import { Obj } from '@v/utils'
import { DocumentDataDescription } from '../../data-description'

interface IProps<P extends Obj> {
  title: string
  dataDescription: DocumentDataDescription<P>
  getMatchers(filter: string): Matcher[]
  onAdd(): void
  onActivate(document: IDocument<string, P>): void
}

export function Catalog<P extends Obj>(props: IProps<P>) {
  const {
    title,
    dataDescription,
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
