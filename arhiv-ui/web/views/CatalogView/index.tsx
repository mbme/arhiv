import * as React from 'react'
import { RouterContext, useLocation } from '@v/web-utils'
import { IDocument, IMatcher } from '../../api'
import { Catalog } from './Catalog'

export const DOCUMENT_TYPE_QUERY_PARAM = 'type'

export function CatalogView() {
  const router = RouterContext.use()

  const location = useLocation()
  const documentType = location.params.find(param => param.name === DOCUMENT_TYPE_QUERY_PARAM)

  const getMatchers = (filter: string): IMatcher[] => [
    { selector: '$.type', pattern: documentType?.value || '' },
    { selector: '$.name', pattern: filter },
  ]

  const onAdd = () => router.push('/documents/new')
  const onActivate = (document: IDocument) => router.push(`/documents/${document.id}`)

  return (
    <Catalog
      title="Documents Catalog"
      getMatchers={getMatchers}
      onAdd={onAdd}
      onActivate={onActivate}
    />
  )
}
