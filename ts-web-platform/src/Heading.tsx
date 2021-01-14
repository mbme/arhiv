import * as React from 'react'
import { Text } from './Text'
import { StyleArg, StyleProps } from './core'

type HeadingVariant = '1' | '2' | '3'

type Props = {
  variant: HeadingVariant,
  children: React.ReactNode,
} & Pick<StyleProps, 'mb'>

function getTag(variant?: HeadingVariant): 'h1' | 'h2' | 'h3' {
  switch(variant) {
    case '3': {
      return 'h1'
    }
    case '2': {
      return 'h2'
    }
    case '1':
    default: {
      return 'h3'
    }
  }
}

function getStyles(props: Props): StyleArg[] {
  return [
    {
      color: 'var(--color-heading)',
      fontWeight: 'bold',
    },
    (!props.variant || props.variant === '1') && {
      fontSize: 'medium',
    },
    props.variant === '2' && {
      fontSize: 'large',
    },
    props.variant === '3' && {
      fontSize: 'xlarge',
    },
  ]
}

export function Heading(props: Props) {
  return (
    <Text
      as={getTag(props.variant)}
      $styles={getStyles(props)}
      mb="medium"
      {...props}
    />
  )
}
