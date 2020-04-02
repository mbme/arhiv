import * as React from 'react'
import {
  Box,
  IProps as IBoxProps,
} from '../Box'
import { Tags } from '../core'

const alignX2Align = {
  left: 'flex-start',
  center: 'center',
  right: 'flex-end',
  stretch: 'stretch',
}

const alignY2Justify = {
  top: 'flex-start',
  center: 'center',
  bottom: 'flex-end',
}

interface IProps<E extends Tags> extends IBoxProps<E> {
  alignX?: keyof typeof alignX2Align
  alignY?: keyof typeof alignY2Justify
}

export function Column<E extends Tags = 'div'>({ alignX = 'center', alignY = 'top', ...props }: IProps<E>) {
  return (
    <Box
      display="flex"
      flexDirection="column"

      alignItems={alignX2Align[alignX]}
      justifyContent={alignY2Justify[alignY]}
      {...props}
    />
  )
}
