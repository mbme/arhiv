import * as React from 'react'
import { usePromise } from '@v/web-utils'
import {
  Box,
  StyleArg,
} from '@v/web-platform'
import { API, IAttachmentSource } from '../api'

const $article: StyleArg = {
  hyphens: 'auto',
  textAlign: 'justify',
}

interface IProps {
  value: string
  newAttachments: IAttachmentSource[]
}

export function Markup({ value, newAttachments }: IProps) {
  const [result] = usePromise(() => (
    API.render_markup({
      value,
      options: {
        newAttachments,
        documentPath: '/document',
      },
    })
  ), [value, newAttachments])

  if (result === undefined) {
    return null
  }

  return (
    <Box
      as="article"
      $style={$article}
      dangerouslySetInnerHTML={{ __html: result }}
    />
  )
}
