import * as React from 'react'
import { Button } from '../Button'
import { ConfirmationDialog } from './ConfirmationDialog'
import { Examples } from '../Examples'

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

const examples = {
  '': (
    <ConfirmationDialogExample />
  ),
}

export function ConfirmationDialogExamples() {
  return (
    <Examples title="Confirmation dialog" examples={examples} />
  )
}
