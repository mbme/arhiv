import * as React from 'react'
import {
  ConfirmationDialog,
} from '@v/web-platform'
import { useActions } from '../../../parts'

interface IProps {
  onConfirmed(): void
}

export function DeleteDocumentButton({ onConfirmed }: IProps) {
  const [isModalVisible, showModal] = React.useState(false)

  useActions(() => [
    {
      onClick: () => showModal(true),
      children: 'Delete Document',
    },
  ], [])

  if (!isModalVisible) {
    return null
  }

  return (
    <ConfirmationDialog
      confirmation="Delete"
      onConfirmed={onConfirmed}
      onCancel={() => showModal(false)}
    >
      Are you sure you want to <b>delete this document?</b>
    </ConfirmationDialog>
  )
}
