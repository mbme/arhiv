import * as React from 'react'
import {
  stylish,
  Image,
  useObservable,
} from '~/web-platform'
import {
  Attachment,
} from '~/arhiv/replica'

const $image = stylish({
  mt: 'medium',
  mb: 'large',
})

interface IProps {
  attachment: Attachment
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
