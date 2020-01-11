import * as React from 'react'
import { AttachFilesButton } from '~/web-platform'
import { createLink } from '~/markup-parser/utils'
import { useArhiv } from '../useArhiv'

interface IProps {
  onAttachments(links: string[]): void
}

export function AddAttachmentsButton({ onAttachments }: IProps) {
  const arhiv = useArhiv()

  const onSelected = async (files: File[]) => {
    const links = files.map(async (file) => {
      const id = await arhiv.attachments.createAttachment(file)

      const attachment = await arhiv.attachments.getAttachment(id)
      if (!attachment) {
        throw new Error(`Can't find new attachment ${id} (file ${file.name})`)
      }

      return createLink(attachment.id, file.name)
    })

    onAttachments(await Promise.all(links))
  }

  return (
    <AttachFilesButton onSelected={onSelected} />
  )
}
