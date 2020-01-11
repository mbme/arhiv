import * as React from 'react'
import { usePromise } from '~/web-platform'
import { useArhiv } from '../../useArhiv'
import { MarkupAttachment } from './MarkupAttachment'

interface IProps {
  link: string
  description: string
}

export function MarkupLink({ link, description }: IProps) {
  const arhiv = useArhiv()

  const [attachment, isReady] = usePromise(() => arhiv.attachments.getAttachment(link), [link])

  if (!isReady) {
    return null
  }

  if (!attachment) {
    return (
      <a href={link} target="_blank" rel="noopener">
        {description || link}
      </a>
    )
  }

  return (
    <MarkupAttachment
      attachment={attachment}
      link={link}
      description={description}
    />
  )
}
