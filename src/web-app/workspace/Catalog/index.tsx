import * as React from 'react'
import { ArhivContext } from '~/web-app/arhiv-context'
import { Column, ProgressLocker } from '~/web-platform'
import { useObservable } from '~/web-utils'
import { matches } from '../document-types'
import { CatalogEntry } from './Entry'

interface IProps {
  filter: string
  openIds: readonly string[]
  openId(id: string): void
}
export function Catalog({ filter, openIds, openId }: IProps) {
  const arhiv = ArhivContext.use()

  const [documents] = useObservable(
    () => arhiv.documents.getDocuments$({ matches: matches(filter) }),
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
