import * as React from 'react'
import {
  stylish,
  Image,
  useObservable,
} from '~/web-platform'
import {
  AttachmentManager,
} from '~/arhiv/replica'

const $image = stylish({
  mt: 'medium',
  mb: 'large',
})

interface IProps {
  attachment: AttachmentManager
  link: string
  description: string
}

export function MarkupAttachment({ attachment, link, description }: IProps) {
  const [blobUrl] = useObservable(() => attachment.getUrl$(), [attachment])

  if (!blobUrl) {
    return null
  }

  if (attachment.attachment._mimeType.startsWith('image/')) {
    return (
      <Image
        $style={$image}
        src={blobUrl}
        alt={description || link}
      />
    )
  }

  return (
    <a href={blobUrl} target="_blank" rel="noopener">
      {description || link}
    </a>
  )
}
