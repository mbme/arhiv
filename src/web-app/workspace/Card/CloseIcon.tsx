import * as React from 'react'
import {
  Icon,
} from '~/web-platform'
import { useWorkspaceManager } from '../useWorkspaceManager'

interface IProps {
  documentId: string
}

export function CloseIcon({ documentId }: IProps) {
  const ws = useWorkspaceManager()

  return (
    <Icon
      type="x"
      title="close"
      onClick={() => ws.closeId(documentId)}
    />
  )
}
