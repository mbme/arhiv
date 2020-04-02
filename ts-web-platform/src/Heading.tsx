import * as React from 'react'
import { Text, IProps as ITextProps } from './Text'
import { StyleProps } from './core'

type Props = ITextProps<'h1'> & Pick<StyleProps, 'mb'>

export function Heading(props: Props) {
  return (
    <Text
      as="h1"
      color="heading"
      fontSize="xlarge"
      bold
      mb="medium"
      {...props}
    />
  )
}
