import * as React from 'react'
import {
  Box,
  IProps as IBoxProps,
} from './Box'

export function Heading(props: IBoxProps) {
  return (
    <Box
      as="h1"
      color="heading"
      fontSize="xlarge"
      bold
      mb="medium"
      {...props}
    />
  )
}
