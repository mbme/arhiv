import * as React from 'react'
import { RouterContext } from '@v/web-utils'
import { Action, Catalog, Frame } from '../../parts'

interface IProps {
  documentType: string
}

export function CatalogView({ documentType }: IProps) {
  const router = RouterContext.use()

  const actions = (
    <Action
      type="action"
      onClick={() => router.push(`/catalog/${documentType}/new`)}
    >
      Add
    </Action>
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
