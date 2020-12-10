import * as React from 'react'
import { Procedure } from '@v/utils'
import { IDocument } from '../../api'
import { Frame, Action, CardData } from '../../parts'

interface IProps {
  document: IDocument
  onEdit: Procedure
  onClose: Procedure
  onMetadata: Procedure
}

export function Card(props: IProps) {
  const {
    document,
    onEdit,
    onClose,
    onMetadata,
  } = props

  const actions = (
    <>
      <Action
        type="action"
        onClick={onClose}
      >
        Close
      </Action>

      <Action
        type="action"
        onClick={onMetadata}
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
      <CardData
        data={document.data}
      />
    </Frame>
  )
}
