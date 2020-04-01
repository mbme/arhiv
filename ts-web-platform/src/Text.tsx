import * as React from 'react'
import {
  useStyles,
  IStyleObject,
  StyleArg,
} from '@v/web-utils'
import { theme } from './style'
import { Tags } from './types'

export interface IProps<E extends Tags> {
  as?: E
  children?: React.ReactNode
  onClick?(e: React.MouseEvent<HTMLElementTagNameMap[E]>): void
  innerRef?: React.RefObject<HTMLElementTagNameMap[E]>

  $style?: StyleArg

  color?: keyof typeof theme.color | string
  bgColor?: keyof typeof theme.color | string

  fontSize?: keyof typeof theme.fontSize | string
  fontFamily?: keyof typeof theme.fontFamily | string

  uppercase?: boolean
  letterSpacing?: string

  bold?: boolean
  fontWeight?: number | string
}

export function Text<E extends Tags = 'div'>({ as, children, onClick, innerRef, $style, ...props }: IProps<E>) {
  const className = useStyles($style, props as IStyleObject)

  return React.createElement(as || 'div', {
    ref: innerRef,
    onClick,
    className,
  }, children)
}
