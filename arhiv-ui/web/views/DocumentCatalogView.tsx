import * as React from 'react'
import { RouterContext } from '@v/web-utils'
import { Catalog } from '../parts'
import { IDocument, IMatcher } from '../api'

export function DocumentCatalogView() {
  const router = RouterContext.use()

  const getMatchers = (filter: string): IMatcher[] => [
    { selector: '$.type', pattern: 'note' },
    { selector: '$.name', pattern: filter },
  ]

  const onAdd = () => router.push('/documents/new')
  const onActivate = (document: IDocument) => router.push(`/documents/${document.id}`)

  return (
    <Catalog
      documentType="note"
      title="Documents Catalog"
      getMatchers={getMatchers}
      onAdd={onAdd}
      onActivate={onActivate}
    />
  )
}
