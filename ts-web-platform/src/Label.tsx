import * as React from 'react'
import { Text, IProps as ITextProps } from './Text'
import {
  Tags,
} from './core'

export function Label<E extends Tags>(props: ITextProps<E>) {
  return (
    <Text
      uppercase
      color="var(--color-secondary)"
      letterSpacing="1.2px"
      fontWeight={500}
      fontSize="small"
      {...props}
    />
  )
}
