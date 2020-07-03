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
      <Box px="large" py="medium">

        <Box mb="medium">
          {children}
        </Box>

        <Row alignX="right">
          <Button onClick={onCancel}>
            CANCEL
          </Button>

          <Button
            variant="primary"
            onClick={onConfirmed}
            $style={{ ml: 'medium' }}
          >
            {confirmation}
          </Button>
        </Row>
      </Box>
    </Modal>
  )
}
