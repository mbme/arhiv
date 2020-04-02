import * as React from 'react'
import { Button } from '../Button'
import { ConfirmationDialog } from './ConfirmationDialog'
import { Example } from '../Example'

export function ConfirmationDialogExamples() {
  const [showModal, setShowModal] = React.useState(false)

  return (
    <Example section title="Confirmation dialog">
      <Button onClick={() => setShowModal(true)}>
        Show modal
      </Button>

      {showModal && (
        <ConfirmationDialog
          confirmation="Remove"
          onConfirmed={() => setShowModal(false)}
          onCancel={() => setShowModal(false)}
        >
          Are you sure you want to <b>remove it?</b>
        </ConfirmationDialog>
      )}
    </Example>
  )
}
