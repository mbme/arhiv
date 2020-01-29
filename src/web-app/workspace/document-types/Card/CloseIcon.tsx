import * as React from 'react'
import { Icon } from '~/web-platform'
import { useWorkspaceURLManager } from '../../useWorkspaceURLManager'

interface IProps {
  documentId: string
}

export function CloseIcon({ documentId }: IProps) {
  const ws = useWorkspaceURLManager()

  return (
    <Icon
      type="x"
      title="close"
      onClick={() => ws.closeId(documentId)}
      $style={{ color: 'secondary' }}
    />
  )
}
