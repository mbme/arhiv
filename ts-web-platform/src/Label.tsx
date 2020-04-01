import * as React from 'react'
import { Tags } from './types'
import { Text, IProps as ITextProps } from './Text'

export function Label<E extends Tags>(props: ITextProps<E>) {
  return (
    <Text
      uppercase
      color="secondary"
      letterSpacing="1.2px"
      fontWeight="500"
      fontSize="small"
      {...props}
    />
  )
}
