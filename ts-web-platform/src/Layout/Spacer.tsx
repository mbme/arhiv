import * as React from 'react'
import {
  Box,
  IProps as IBoxProps,
} from '../Box'

export function Spacer(props: IBoxProps<'div'>) {
  return (
    <Box
      flex="1 1 auto"
      {...props}
    />
  )
}
