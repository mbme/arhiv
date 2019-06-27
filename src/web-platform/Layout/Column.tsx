import * as React from 'react'
import {
  Box,
  IProps as IBoxProps,
} from '../Box'

interface IProps extends IBoxProps {
  alignX?: 'left' | 'center' | 'right'
  alignY?: 'top' | 'center' | 'bottom'
}

const alignX2Align = {
  left: 'flex-start',
  center: 'center',
  right: 'flex-end',
}

const alignY2Justify = {
  top: 'flex-start',
  center: 'center',
  bottom: 'flex-end',
}

export function Column({ alignX = 'center', alignY = 'top', ...props }: IProps) {
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
