import * as React from 'react'
import { RouterContext } from '@v/web-platform'
import {
  Catalog,
  CatalogOptionsOverrides,
  FrameTitle,
  useActions,
} from '../../parts'
import { useDataDescription } from '../../data-manager'

interface IProps {
  documentType: string
  skipAddDocumentAction?: boolean
  catalogOptions?: CatalogOptionsOverrides
}

export function CatalogView({ documentType, skipAddDocumentAction, catalogOptions }: IProps) {
  const router = RouterContext.use()

  const {
    mandatoryFields,
  } = useDataDescription(documentType)

  useActions(() => [
    {
      onClick: () => router.push('/'),
      children: 'Close',
    },
    (mandatoryFields.length === 0 && !skipAddDocumentAction) ? {
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
        options={catalogOptions}
      />
    </>
  )
}
