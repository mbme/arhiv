import * as React from 'react'
import {
  Box,
  IProps as IBoxProps,
} from './Box'

export function Text(props: IBoxProps) {
  return (
    <Box
      as="span"
      display="inline-block"
      {...props}
    />
  )
}
