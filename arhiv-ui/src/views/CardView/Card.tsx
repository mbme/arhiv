import { RouterContext } from '@v/web-utils'
import * as React from 'react'
import { IDocument } from '@v/arhiv-api'
import { useDataDescription } from '../../data-manager'
import {
  CardData,
  Catalog,
  CatalogOptionsOverrides,
  useActions,
} from '../../parts'

interface IProps {
  document: IDocument
  catalogOptions?: CatalogOptionsOverrides
}

export function Card({ document, catalogOptions }: IProps) {
  const router = RouterContext.use()

  const documentType = document.documentType
  const { dataDescription } = useDataDescription(documentType)
  const childDocumentType = dataDescription.collectionOf?.itemType

  useActions(() => [
    {
      onClick: () => router.goBack(),
      children: 'Close',
    },
    childDocumentType ? {
      onClick() {
        router.push({
          path: `/documents/${childDocumentType}/new`,
          params: [{ name: documentType, value: document.id }],
        })
      },
      children: `Add ${childDocumentType}`,
    } : undefined,
    {
      onClick: () => router.replace(`/documents/${document.id}/metadata`),
      children: 'Show Metadata',
    },
    {
      onClick: () => router.replace(`/documents/${document.id}/edit`),
      children: `Edit ${documentType}`,
    },
  ], [document.id])

  return (
    <>
      <CardData
        documentType={documentType}
        data={document.data}
      />

      {childDocumentType && (
        <Catalog
          key={childDocumentType}
          documentType={childDocumentType}
          collectionMatcher={{ Field: { selector: `$.${documentType}`, pattern: document.id }}}
          options={catalogOptions}
        />
      )}
    </>
  )
}
