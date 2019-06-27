import * as React from 'react'
import { Button } from '../Button'
import { Modal } from './Modal'
import { Box } from '../Box'
import { Row } from '../Layout'

interface IProps {
  children: React.ReactNode
  confirmation: React.ReactNode
  onConfirmed(): void
  onCancel(): void
}

export function ConfirmationDialog({ children, confirmation, onConfirmed, onCancel }: IProps) {
  return (
    <Modal onCancel={onCancel}>
      <Box mb="medium">
        {children}
      </Box>

      <Row alignX="right">
        <Button onClick={onCancel}>
          CANCEL
        </Button>

        <Button
          primary
          onClick={onConfirmed}
          $style={{ ml: 'medium' }}
        >
          {confirmation}
        </Button>
      </Row>
    </Modal>
  )
}

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
