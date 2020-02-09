import * as React from 'react'
import { Icon } from '~/web-platform'
import { useWorkspaceStore } from '~/web-app/workspace-store'

interface IProps {
  documentId: string
}

export function CloseIcon({ documentId }: IProps) {
  const store = useWorkspaceStore()

  return (
    <Icon
      type="x"
      title="close"
      onClick={() => store.closeDocument(documentId)}
      $style={{ color: 'secondary' }}
    />
  )
}
