import * as React from 'react'
import {
  useObservable,
  ProgressLocker,
  Column,
} from '~/web-platform'
import { useArhiv } from '~/arhiv/useArhiv'
import { CatalogEntry } from './Entry'

interface IProps {
  filter: string
  openIds: readonly string[]
  openId(id: string): void
}
export function Catalog({ filter, openIds, openId }: IProps) {
  const arhiv = useArhiv()

  const [documents] = useObservable(
    () => arhiv.documents.getDocuments$({ filter }),
    [filter],
  )

  if (!documents) {
    return (
      <ProgressLocker />
    )
  }

  const items = documents
    .map(document => (
      <CatalogEntry
        key={document.id}
        document={document}
        isOpen={openIds.includes(document.id)}
        onClick={() => openId(document.id)}
      />
    ))

  return (
    <Column
      width="500px"
      maxWidth="100%"
      alignX="stretch"
      mx="auto"
    >
      {items}
    </Column>
  )
}
