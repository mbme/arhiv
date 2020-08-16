import * as React from 'react'
import {
  ConfirmationDialog,
} from '@v/web-platform'
import { Action } from '../parts'

interface IProps {
  onConfirmed(): void
}

export function DeleteDocumentButton({ onConfirmed }: IProps) {
  const [isModalVisible, showModal] = React.useState(false)

  return (
    <>
      <Action
        type="action"
        onClick={() => showModal(true)}
      >
        Delete
      </Action>

      {isModalVisible && (
        <ConfirmationDialog
          confirmation="Delete"
          onConfirmed={onConfirmed}
          onCancel={() => showModal(false)}
        >
          Are you sure you want to <b>delete this document?</b>
        </ConfirmationDialog>
      )}
    </>
  )
}
