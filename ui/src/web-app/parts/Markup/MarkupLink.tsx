import * as React from 'react'
import { usePromise } from '~/web-utils'
import { ArhivContext } from '~/web-app/arhiv-context'
import { MarkupAttachment } from './MarkupAttachment'

interface IProps {
  link: string
  description: string
}

export function MarkupLink({ link, description }: IProps) {
  const arhiv = ArhivContext.use()

  const [attachment] = usePromise(() => arhiv.attachments.getAttachment(link), [link])

  if (!attachment) {
    return (
      <a href={link} target="_blank" rel="noopener noreferrer">
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
