import * as React from 'react'
import { IAttachment } from '../../notes'
import { Image, StyleArg } from '@v/web-platform'
import { useObservable } from '@v/web-utils'

const $image: StyleArg = {
  mt: 'medium',
  mb: 'large',
}

interface IProps {
  attachment: IAttachment
  link: string
  description: string
}

export function MarkupAttachment({ attachment, link, description }: IProps) {
  const [blobUrl] = useObservable(() => attachment.getUrl$(), [attachment])

  if (!blobUrl) {
    return null
  }

  if (attachment.attachment.mimeType.startsWith('image/')) {
    return (
      <Image
        $style={$image}
        src={blobUrl}
        alt={description || link}
      />
    )
  }

  return (
    <a href={blobUrl} target="_blank" rel="noopener noreferrer">
      {description || link}
    </a>
  )
}
