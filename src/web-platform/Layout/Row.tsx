import * as React from 'react'
import {
  Box,
  IProps as IBoxProps,
} from '../Box'

interface IProps extends IBoxProps {
  alignX?: 'left' | 'center' | 'right'
  alignY?: 'top' | 'center' | 'bottom'
}

const alignX2Justify = {
  left: 'flex-start',
  center: 'center',
  right: 'flex-end',
}

const alignY2Align = {
  top: 'flex-start',
  center: 'center',
  bottom: 'flex-end',
}

export function Row({ alignX = 'center', alignY = 'center', ...props }: IProps) {
  return (
    <Box
      display="flex"
      alignItems={alignY2Align[alignY]}
      justifyContent={alignX2Justify[alignX]}
      {...props}
    />
  )
}
