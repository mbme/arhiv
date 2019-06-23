import * as React from 'react'
import {
  theme,
  stylish,
} from '~/web-platform'
import {
  Markup,
} from '../parts'

export const $title = stylish({
  textAlign: 'center',
  letterSpacing: '1.4px',
  fontWeight: 'bold',
  fontSize: theme.fontSize.large,

  marginBottom: theme.spacing.medium,
})

interface IProps {
  name: string
  data: string
}

export function Note({ name, data }: IProps) {
  return (
    <>
      <h1 className={$title.className}>
        {name}
      </h1>

      <Markup value={data} />
    </>
  )
}
