import * as React from 'react'
import { RouterContext } from '@v/web-utils'
import { IDocument } from '../../api'
import { Catalog } from './Catalog'

interface IProps {
  documentType: string
}

export function CatalogView({ documentType }: IProps) {
  const router = RouterContext.use()

  const onAdd = () => router.push(`/catalog/${documentType}/new`)
  const onActivate = (document: IDocument) => router.push(`/documents/${document.id}`)

  return (
    <Catalog
      key={documentType}
      documentType={documentType}
      title={`${documentType} Catalog`}
      onAdd={onAdd}
      onActivate={onActivate}
    />
  )
}
