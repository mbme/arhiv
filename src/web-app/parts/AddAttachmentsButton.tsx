import * as React from 'react'
import { useArhiv } from '~/arhiv'
import { AttachFilesButton } from '~/web-platform'
import { createLink } from '~/markup-parser/utils'

interface IProps {
  onAttachments(links: string[]): void
}

export function AddAttachmentsButton({ onAttachments }: IProps) {
  const arhiv = useArhiv()

  const onSelected = (files: File[]) => {
    const links = files.map((file) => {
      const id = arhiv.attachments.createAttachment(file)
      const attachment = arhiv.attachments.getAttachment(id)
      if (!attachment) {
        throw new Error(`Can't find new attachment ${id} (file ${file.name})`)
      }

      return createLink(attachment.url, file.name)
    })

    onAttachments(links)
  }

  return (
    <AttachFilesButton onSelected={onSelected} />
  )
}
