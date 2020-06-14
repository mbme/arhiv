import * as React from 'react'
import { IAttachment, API } from '../../notes'
import { Image, StyleArg } from '@v/web-platform'
import { usePromise } from '@v/web-utils'

const IMAGE_EXT = [
  '.png',
  '.jpg',
  '.jpeg',
  '.svg',
]

function isImageFileName(filename: string): boolean {
  return IMAGE_EXT.some(ext => filename.endsWith(ext))
}

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
  const [url] = usePromise(() => API.get_attachment_url(attachment.id), [attachment.id])

  if (!url) {
    return null
  }

  if (isImageFileName(attachment.filename)) {
    return (
      <Image
        $style={$image}
        src={url}
        alt={description || link}
      />
    )
  }

  return (
    <a href={url} target="_blank" rel="noopener noreferrer">
      {description || link}
    </a>
  )
}
