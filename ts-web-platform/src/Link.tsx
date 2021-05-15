import * as React from 'react'
import { Link as RouterLink } from './utils'
import {
  StyleArg,
  useStyles,
} from './core'

const $cleanLink: StyleArg = {
  color: 'inherit',
  '&:hover': {
    color: 'inherit',
  },
}

interface IProps extends Omit<React.ComponentProps<typeof RouterLink>, 'className'> {
  $style?: StyleArg
  $styles?: StyleArg[]
  clean?: boolean
}

export function Link({ $styles = [], $style, clean, ...props }: IProps) {
  const ref = React.useRef<HTMLAnchorElement>(null)

  const className = useStyles(
    clean && $cleanLink,
    ...$styles,
    $style,
  )

  return  (
    <RouterLink
      ref={ref}
      className={className}
      {...props}
    />
  )
}
