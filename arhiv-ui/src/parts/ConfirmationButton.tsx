import * as React from 'react'
import {
  ConfirmationDialog,
} from '@v/web-platform'
import { useActions } from '.'

interface IProps {
  name: string
  prompt: React.ReactNode
  confirmation: string
  onConfirmed(): void
}

export function ConfirmationButton({ name, prompt, confirmation, onConfirmed }: IProps) {
  const [isModalVisible, showModal] = React.useState(false)

  useActions(() => [
    {
      onClick: () => showModal(true),
      children: name,
    },
  ], [])

  if (!isModalVisible) {
    return null
  }

  return (
    <ConfirmationDialog
      confirmation={confirmation}
      onConfirmed={onConfirmed}
      onCancel={() => showModal(false)}
    >
      {prompt}
    </ConfirmationDialog>
  )
}
