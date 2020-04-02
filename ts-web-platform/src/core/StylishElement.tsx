import * as React from 'react'
import { useStyles } from './hooks'
import {
  StyleArg,
  Tags,
} from './types'

interface IStylishProps<E extends Tags = 'div'> extends React.HTMLProps<HTMLElementTagNameMap[E]> {
  as?: E
  $styles?: StyleArg[]
  innerRef?: React.RefObject<HTMLElementTagNameMap[E]>
  ref?: undefined
}

export function StylishElement<E extends Tags = 'div'>({ $styles = [], as, innerRef, ...props }: IStylishProps<E>) {
  const className = useStyles(...$styles)

  return React.createElement(as || 'div', {
    ref: innerRef,
    className,
    ...props
  })
}
