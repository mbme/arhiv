import * as React from 'react'
import {
  Box,
  IProps as IBoxProps,
} from './Box'

export function Label(props: IBoxProps) {
  return (
    <Box
      uppercase
      color="secondary"
      letterSpacing="1.2px"
      fontWeight="500"
      fontSize="small"
      {...props}
    />
  )
}
