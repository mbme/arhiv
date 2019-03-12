import * as React from 'react'
import {
  flexRow,
  margin,
  section,
} from '../styles'
import { Button } from '../Button'
import { Modal } from './Modal'

interface IProps {
  children: React.ReactNode
  confirmation: React.ReactNode
  onConfirmed(): void
  onCancel(): void
}

export function ConfirmationDialog({ children, confirmation, onConfirmed, onCancel }: IProps) {
  return (
    <Modal onCancel={onCancel}>
      <div className={section}>
        {children}
      </div>

      <div className={flexRow('flex-end')}>
        <Button onClick={onCancel}>
          CANCEL
        </Button>

        <Button
          primary
          onClick={onConfirmed}
          className={margin({ left: 'medium' })}
        >
          {confirmation}
        </Button>
      </div>
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
