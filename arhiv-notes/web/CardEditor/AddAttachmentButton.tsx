import * as React from 'react'
import { Icon } from '@v/web-platform'
import { createLink } from '../markup-parser'
import { API } from '../notes'

interface IProps {
  onAttachments(links: string[]): void
}

export function AddAttachmentButton({ onAttachments }: IProps) {
  const selectFiles = async () => {
    const attachments = await API.pick_attachments()

    onAttachments(attachments.map(attachment => createLink(attachment.id, attachment.filename)))
  }

  return (
    <Icon
      title="Attach files"
      type="paperclip"
      onClick={selectFiles}
    />
  )
}
