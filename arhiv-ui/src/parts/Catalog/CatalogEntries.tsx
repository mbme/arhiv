import * as React from 'react'
import { partitionBy } from '@v/utils'
import {
  Accordion,
  Box,
  Text,
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
    <React.Fragment key={item.document.id}>
      <CatalogEntry
        document={item.document}
        preview={item.preview}
        showModificationDate={uiOptions.showEntryModificationDate}
        showDataFields={uiOptions.showEntryDataFields}
      />

      <Box
        as="hr"
        my="medium"
        mx="medium"
        borderColor="var(--color-link)"
      />
    </React.Fragment>
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
    .map((group) => {
      const groupName = group[0].document.data[fieldName]

      return (
        <Box
          key={group[0].document.data[fieldName]}
          mt="medium"
          mb="large"
        >
          <Accordion
            defaultOpen={uiOptions.openGroups.includes(groupName)}
            summary={(
              <Text
                fontSize="large"
                bold
                as="span"
              >
                {groupName} ({group.length})
              </Text>
            )}
          >
            {group.map(renderItem)}
          </Accordion>
        </Box>
      )
    })

  return (
    <>
      {entries}
    </>
  )
}
