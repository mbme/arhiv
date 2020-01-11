import * as React from 'react'
import { ChronoFormatter } from '~/chrono'
import {
  useObservable,
  FilterInput,
  Box,
  ProgressLocker,
  theme,
  Label,
} from '~/web-platform'
import { useWorkspaceManager } from './useWorkspaceManager'
import { useArhiv } from '../useArhiv'

const dateFormat = new ChronoFormatter('YYYY/MM/DD')

export function DeckOfCards() {
  const ws = useWorkspaceManager()
  const arhiv = useArhiv()

  const [documents] = useObservable(() => arhiv.documents.getDocuments$())

  if (!documents) {
    return (
      <ProgressLocker />
    )
  }

  const items = documents
    .filter(document => document.matches(ws.filter))
    .map(document => (
      <Box
        key={document.id}
        onClick={() => ws.openId(document.id)}
        mb="medium"
        ml="small"
        cursor="pointer"
        borderRight="5px solid white"
        borderRightColor={ws.openIds.includes(document.id) ? theme.color.highlight : undefined}
      >
        <Label
          fontSize="fine"
        >
          {document.type}
        </Label>

        {document.getTitle()}

        <Box
          as="small"
          display="block"
        >
          {dateFormat.format(document.updatedAt)}
        </Box>
      </Box>
    ))

  return (
    <Box
      width="360px"
      height="100%"
      overflowY="scroll"
      ml="medium"
    >
      <Box
        position="sticky"
        top="0"
        background={theme.color.bg}
        py="fine"
      >
        <FilterInput
          placeholder="Filter notes"
          filter={ws.filter}
          onChange={newFilter => ws.updateFilter(newFilter)}
          alwaysExpanded
        />
      </Box>

      <Box
        as="small"
        pl="medium"
        mb="medium"
        display="block"
      >
        {items.length} items
      </Box>

      {items}
    </Box>
  )
}
