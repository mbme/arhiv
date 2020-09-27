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
  const [isFocused, setRef] = useFocusable<HTMLAnchorElement>()

  const className = useStyles(
    isFocused && {
      border: '1px solid red',
    },
    clean && $cleanLink,
    $style,
  )

  return  (
    <RouterLink
      ref={setRef}
      className={className}
      {...props}
    />
  )
}
