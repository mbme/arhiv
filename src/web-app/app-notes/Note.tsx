import * as React from 'react'
import { style } from 'typestyle'
import { theme } from '~/web-components'
import {
  Markup,
} from '../parts'

export const titleStyle = style({
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
      <h1 className={titleStyle}>
        {name}
      </h1>

      <Markup value={data} />
    </>
  )
}
