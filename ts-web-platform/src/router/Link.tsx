import * as React from 'react'
import {
  SimpleLocation,
} from './types'
import { RouterContext } from './context'
import { StyleArg, useStyles } from '../core'

const $cleanLink: StyleArg = {
  color: 'inherit',
  '&:hover': {
    color: 'inherit',
  },
}

interface IProps {
  to: SimpleLocation
  newTab?: boolean
  $style?: StyleArg
  $styles?: StyleArg[]
  clean?: boolean
  children: React.ReactNode
}

export const Link = React.forwardRef(
  function Link({ to, newTab, $style, $styles = [], clean, children }: IProps, ref: React.Ref<HTMLAnchorElement>) {
    const router = RouterContext.use()

    const url = router.getUrl(to)

    const className = useStyles(
      clean && $cleanLink,
      ...$styles,
      $style,
    )

    const onClick = (e: React.MouseEvent<HTMLAnchorElement>) => {
      router.push(to)
      e.preventDefault()
    }

    return (
      <a
        ref={ref}
        href={url}
        onClick={onClick}
        target={newTab ? '_blank' : undefined}
        rel={newTab ? 'noopener' : undefined}
        className={className}
      >
        {children}
      </a>
    )
  },
)
