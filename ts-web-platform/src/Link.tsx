import * as React from 'react'
import { Link as RouterLink } from '@v/web-utils'
import {
  StyleArg,
  useStyles,
} from './core'
import { useFocusable } from './Focus'

const $cleanLink: StyleArg = {
  color: 'inherit',
  '&:hover': {
    color: 'inherit',
  },
}

interface IProps extends Omit<React.ComponentProps<typeof RouterLink>, 'className'> {
  $style?: StyleArg
  clean?: boolean
}

export function Link({ $style, clean, ...props }: IProps) {
  const ref = React.useRef<HTMLAnchorElement>(null)
  const isSelected = useFocusable(ref)

  const className = useStyles(
    isSelected && {
      border: '1px solid red',
    },
    clean && $cleanLink,
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
