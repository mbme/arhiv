import * as React from 'react'
import {
  Box,
  IProps as IBoxProps,
} from '../Box'
import { Tags } from '../core'

interface IProps<E extends Tags> extends IBoxProps<E> {
  alignX?: 'left' | 'center' | 'right' | 'space-between'
  alignY?: 'top' | 'center' | 'bottom'
}

const alignX2Justify = {
  left: 'flex-start',
  center: 'center',
  right: 'flex-end',
  'space-between': 'space-between',
}

const alignY2Align = {
  top: 'flex-start',
  center: 'center',
  bottom: 'flex-end',
}

export function Row<E extends Tags = 'div'>({ alignX = 'center', alignY = 'center', ...props }: IProps<E>) {
  return (
    <Box
      display="flex"
      alignItems={alignY2Align[alignY]}
      justifyContent={alignX2Justify[alignX]}
      {...props}
    />
  )
}
