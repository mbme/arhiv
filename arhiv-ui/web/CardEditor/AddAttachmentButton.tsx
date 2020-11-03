import * as React from 'react'
import { API, createLink } from '../api'
import { Action } from '../parts'

interface IProps {
  onAttachments(links: string[]): void
}

export function AddAttachmentButton({ onAttachments }: IProps) {
  const selectFiles = async () => {
    const attachments = await API.pick_attachments()

    onAttachments(attachments.map(attachment => createLink(attachment.id, attachment.filename)))
  }

  return (
    <Action
      type="action"
      onClick={selectFiles}
    >
      Attach File
    </Action>
  )
}
