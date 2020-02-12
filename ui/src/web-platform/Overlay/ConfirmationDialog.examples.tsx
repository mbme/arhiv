import * as React from 'react'
import { Button } from '../Button'
import { ConfirmationDialog } from './ConfirmationDialog'

const ConfirmationDialogExample = () => {
  const [showModal, setShowModal] = React.useState(false)

  return (
    <>
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
    </>
  )
}

export const examples = {
  '': (
    <ConfirmationDialogExample />
  ),
}
