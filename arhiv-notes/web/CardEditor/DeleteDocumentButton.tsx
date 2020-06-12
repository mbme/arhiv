import * as React from 'react'
import {
  Icon,
  ConfirmationDialog,
} from '@v/web-platform'

interface IProps {
  onConfirmed(): void
}

export function DeleteDocumentButton({ onConfirmed }: IProps) {
  const [isModalVisible, showModal] = React.useState(false)

  return (
    <>
      <Icon
        title="Delete document"
        type="trash-2"
        onClick={() => showModal(true)}
      />

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
