import * as React from 'react'
import { usePromise } from '@v/web-utils'
import {
  Box,
  StyleArg,
} from '@v/web-platform'
import { API } from '../../api'

const $article: StyleArg = {
  hyphens: 'auto',
  textAlign: 'justify',
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
