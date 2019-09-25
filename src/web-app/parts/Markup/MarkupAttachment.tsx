import * as React from 'react'
import {
  stylish,
  Image,
} from '~/web-platform'
import {
  Attachment,
} from '~/arhiv'

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
  const [blobUrl, setBlobUrl] = React.useState<string | undefined>(undefined)
  React.useEffect(() => {
    const url$ = attachment.getUrl$()

    url$.subscribe({
      next: setBlobUrl,
    })

    return url$.complete
  }, [attachment.id])

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
