import * as React from 'react'
import { Procedure } from '@v/utils'
import { IDocument } from '../../api'
import { Frame, Action, CardData } from '../../parts'
import { Metadata } from './Metadata'

interface IProps {
  document: IDocument
  onEdit: Procedure
  onClose: Procedure
}

export function Card(props: IProps) {
  const {
    document,
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
        />
      )}
    </Frame>
  )
}
