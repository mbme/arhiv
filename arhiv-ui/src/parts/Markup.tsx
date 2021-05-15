import * as React from 'react'
import {
  Box,
  StyleArg,
  usePromise,
} from '@v/web-platform'
import { API } from '@v/arhiv-api'

const $article: StyleArg = {
  hyphens: 'auto',
  textAlign: 'justify',
  overflowWrap: 'break-word',
}

interface IProps {
  value: string
}

export function Markup({ value }: IProps) {
  const [result] = usePromise(() => API.render_markup(value), [value])

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
