import * as React from 'react'
import { Obj, Procedure } from '@v/utils'
import { IDocument } from '../api'
import { Frame, Action } from '../parts'
import { Metadata } from './Metadata'
import { DocumentDataDescription } from '../data-description'
import { CardData } from './CardData'

interface IProps<P extends Obj> {
  document: IDocument<string, P>
  dataDescription: DocumentDataDescription<P>
  onEdit: Procedure
  onClose: Procedure
}

export function Card<P>(props: IProps<P>) {
  const {
    document,
    dataDescription,
    onEdit,
    onClose,
  } = props

  const [metadata, showMetadata] = React.useState(false)

  const actions = metadata ? (
    <Action
      type="action"
      onClick={() => showMetadata(false)}
    >
      Back
    </Action>
  ) : (
    <>
      <Action
        type="action"
        onClick={onClose}
      >
        Close
      </Action>

      <Action
        type="action"
        onClick={() => showMetadata(true)}
      >
        Show Metadata
      </Action>

      <Action
        type="action"
        onClick={onEdit}
      >
        Edit Document
      </Action>
    </>
  )

  return (
    <Frame
      actions={actions}
      title="Card"
    >
      {metadata ? (
        <Metadata document={document} />
      ) : (
        <CardData
          data={document.data}
          dataDescription={dataDescription}
        />
      )}
    </Frame>
  )
}
