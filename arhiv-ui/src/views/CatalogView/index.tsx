import * as React from 'react'
import { RouterContext } from '@v/web-utils'
import { Catalog, FrameTitle, useActions } from '../../parts'
import { useDataDescription } from '../../data-manager'

interface IProps {
  documentType: string
}

export function CatalogView({ documentType }: IProps) {
  const router = RouterContext.use()

  const {
    mandatoryFields,
    uiOptions,
  } = useDataDescription(documentType)

  useActions(() => [
    {
      onClick: () => router.push('/'),
      children: 'Close',
    },
    (mandatoryFields.length === 0 && !uiOptions.catalog.skipAddDocumentAction) ? {
      onClick: () => router.push(`/documents/${documentType}/new`),
      children: `Add ${documentType}`,
    } : undefined,
  ])

  return (
    <>
      <FrameTitle>
        {documentType} Catalog
      </FrameTitle>

      <Catalog
        key={documentType}
        documentType={documentType}
      />
    </>
  )
}
