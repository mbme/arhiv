import React from 'react'
import {
  Icon,
  ConfirmationDialog,
} from '~/web-platform'

interface IProps {
  onConfirmed(): void
}

export function DeleteNoteButton({ onConfirmed }: IProps) {
  const [isModalVisible, showModal] = React.useState(false)

  return (
    <>
      <Icon
        title="Delete note"
        type="trash-2"
        onClick={() => showModal(true)}
      />

      {isModalVisible && (
        <ConfirmationDialog
          confirmation="Delete"
          onConfirmed={onConfirmed}
          onCancel={() => showModal(false)}
        >
          Are you sure you want to <b>delete this note?</b>
        </ConfirmationDialog>
      )
      }
    </>
  )
}
