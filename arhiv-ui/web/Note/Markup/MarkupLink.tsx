import * as React from 'react'
import { usePromise } from '@v/web-utils'
import { MarkupAttachment } from './MarkupAttachment'
import { API } from '../../api'

interface IProps {
  link: string
  description: string
}

export function MarkupLink({ link, description }: IProps) {
  const [attachment] = usePromise(() => API.get_attachment(link), [link])

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
