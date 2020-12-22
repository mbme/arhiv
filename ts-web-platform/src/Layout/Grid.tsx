import * as React from 'react'
import {
  Box,
  IProps as IBoxProps,
} from '../Box'
import { Tags } from '../core'

export function Grid<E extends Tags = 'div'>(props: IBoxProps<E>) {
  return (
    <Box
      display="grid"
      gridTemplateColumns="repeat(auto-fill, minmax(200px, 1fr) )"
      gridGap="0.8rem"
      {...props}
    />
  )
}
