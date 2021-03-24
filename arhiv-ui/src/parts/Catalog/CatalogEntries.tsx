import * as React from 'react'
import { partitionBy } from '@v/utils'
import {
  Box,
  Heading,
} from '@v/web-platform'
import { CatalogEntry } from './CatalogEntry'
import { IDocumentExt } from '../../api'
import { ICatalogOptions } from './options'

function createRule(field: string) {
  let lastVal: any = undefined
  return (item: IDocumentExt): boolean => {
    const currentVal = item.document.data[field]
    const result = currentVal !== lastVal
    lastVal = currentVal

    return result
  }
}

interface IProps {
  items: IDocumentExt[]
  uiOptions: ICatalogOptions
}

export function CatalogEntries({ items, uiOptions }: IProps) {
  const renderItem = (item: IDocumentExt) => (
    <CatalogEntry
      key={item.document.id}
      document={item.document}
      preview={item.preview}
      showModificationDate={uiOptions.showEntryModificationDate}
      showDataFields={uiOptions.showEntryDataFields}
    />
  )

  if (uiOptions.groupByField === undefined) {
    return (
      <>
        {items.map(renderItem)}
      </>
    )
  }

  const fieldName: string = uiOptions.groupByField

  const entries = partitionBy(createRule(fieldName), items)
    .map(group => (
      <Box mb="large" key={group[0].document.data[fieldName]}>
        <Heading variant="1">
          {group[0].document.data[fieldName]} ({group.length})
        </Heading>

        {group.map(renderItem)}
      </Box>
    ))

  return (
    <>
      {entries}
    </>
  )
}
