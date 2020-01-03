import * as React from 'react'
import {
  formatTs,
  fuzzySearch,
} from '~/utils'
import { useRouter } from '~/web-router'
import { useArhiv } from '~/arhiv/replica'
import {
  useObservable,
  stylish,
  Button,
  FilterInput,
  CleanLink,
  Link,
  Box,
  Spacer,
  ProgressLocker,
} from '~/web-platform'

interface IProps {
  filter: string,
}

export function DocumentsList({ filter }: IProps) {
  const router = useRouter()
  const arhiv = useArhiv()

  const [documents, isReady] = useObservable(() => arhiv.documents.getDocuments$())

  if (!isReady) {
    return (
      <ProgressLocker />
    )
  }

  const items = (documents || [])
    .filter(document => fuzzySearch(filter, document.id))
    .map(document => (
      <Box key={document.id}>
        <Box as="small" mr="small">
          {formatTs(note.record._updatedTs)}
        </Box>

        {document.record._type} {document.id}
      </Box>
    ))

  return (
    <div>
    </div>
  )
}
