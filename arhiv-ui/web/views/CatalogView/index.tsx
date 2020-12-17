import * as React from 'react'
import { RouterContext } from '@v/web-utils'
import { Action, Catalog, Frame } from '../../parts'
import { useDataDescription } from '../../data-manager'

interface IProps {
  documentType: string
}

export function CatalogView({ documentType }: IProps) {
  const router = RouterContext.use()

  const { mandatoryFields } = useDataDescription(documentType)

  const actions = (
    <>
      {mandatoryFields.length === 0 && (
        <Action
          type="action"
          onClick={() => router.push(`/documents/${documentType}/new`)}
        >
          Add {documentType}
        </Action>
      )}
    </>
  )

  return (
    <Frame
      actions={actions}
      title={`${documentType} Catalog`}
    >
      <Catalog
        key={documentType}
        documentType={documentType}
      />
    </Frame>
  )
}
