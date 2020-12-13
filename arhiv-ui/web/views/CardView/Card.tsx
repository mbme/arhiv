import { RouterContext } from '@v/web-utils'
import * as React from 'react'
import { IDocument } from '../../api'
import { Frame, Action, CardData, Catalog } from '../../parts'

interface IProps {
  document: IDocument
}

export function Card({ document }: IProps) {
  const router = RouterContext.use()

  const documentType = document.data.type

  const actions = (
    <>
      <Action
        type="action"
        onClick={() => router.goBack()}
      >
        Close
      </Action>

      <Action
        type="action"
        onClick={() => router.replace(`/documents/${document.id}/metadata` )}
      >
        Show Metadata
      </Action>

      <Action
        type="action"
        onClick={() => router.replace(`/documents/${document.id}/edit` )}
      >
        Edit Document
      </Action>
    </>
  )

  return (
    <Frame
      actions={actions}
      title={`${documentType} Card`}
    >
      <CardData
        data={document.data}
      />

      <Catalog
        key={documentType}
        documentType={documentType}
        collectionMatcher={{ selector: `$.${documentType}`, pattern: document.id, fuzzy: false }}
      />
    </Frame>
  )
}
