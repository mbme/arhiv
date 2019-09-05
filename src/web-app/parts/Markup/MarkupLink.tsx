import * as React from 'react'
import { useArhiv } from '~/arhiv'
import { MarkupAttachment } from './MarkupAttachment'

interface IProps {
  link: string
  description: string
}

export function MarkupLink({ link, description }: IProps) {
  const arhiv = useArhiv()

  const attachment = arhiv.attachments.getAttachment(link)

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
