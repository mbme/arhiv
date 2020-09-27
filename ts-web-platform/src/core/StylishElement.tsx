import * as React from 'react'
import { useStyles } from './hooks'
import {
  StyleArg,
  Tags,
} from './types'

interface IProps<E extends Tags = 'div'> extends React.HTMLProps<HTMLElementTagNameMap[E]> {
  as?: E
  $styles?: StyleArg[]
}

export const StylishElement = React.forwardRef(
  function StylishElement<E extends Tags = 'div'>(
    { $styles = [], as, ...props }: IProps<E>,
    ref: React.Ref<HTMLElementTagNameMap[E]>,
  ) {
    const className = useStyles(...$styles)

    return React.createElement(as || 'div', {
      ref,
      className,
      ...props
    })
  },
)
