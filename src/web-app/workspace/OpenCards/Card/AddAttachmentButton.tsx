import * as React from 'react'
import { createLink } from '~/arhiv/markup-parser'
import { ArhivContext } from '~/web-app/arhiv-context'
import { FilePicker, Icon } from '~/web-platform'

interface IProps {
  onAttachments(links: string[]): void
}

export function AddAttachmentsButton({ onAttachments }: IProps) {
  const arhiv = ArhivContext.use()
  const filePickerRef = React.useRef<FilePicker>(null)

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

  const openFilePicker = () => {
    filePickerRef.current!.open()
  }

  return (
    <>
      <Icon
        title="Attach files"
        type="paperclip"
        onClick={openFilePicker}
      />

      <FilePicker
        ref={filePickerRef}
        onSelected={onSelected}
      />
    </>
  )
}
