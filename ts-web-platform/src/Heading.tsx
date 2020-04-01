import * as React from 'react'
import { Text, IProps as ITextProps } from './Text'
import { theme } from './style'

interface IProps extends ITextProps<'h1'> {
  mb?: keyof typeof theme.spacing | string
}

export function Heading(props: IProps) {
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
