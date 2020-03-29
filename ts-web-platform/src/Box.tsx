import * as React from 'react'
import { useStyles, IStyleObject } from '@v/web-utils'
import { theme } from './style'

type Tags = keyof HTMLElementTagNameMap
type Spacing = keyof typeof theme.spacing | string

interface IProps<E extends Tags> {
  as?: E
  children?: React.ReactNode
  onClick?(e: React.MouseEvent<HTMLElementTagNameMap[E]>): void
  innerRef?: React.RefObject<HTMLElementTagNameMap[E]>

  relative?: boolean
  absolute?: boolean
  hidden?: boolean

  m?: Spacing
  mx?: Spacing
  my?: Spacing
  mt?: Spacing
  mb?: Spacing
  ml?: Spacing
  mr?: Spacing

  p?: Spacing
  px?: Spacing
  py?: Spacing
  pt?: Spacing
  pb?: Spacing
  pl?: Spacing
  pr?: Spacing

  width?: Spacing
  height?: Spacing

  top?: Spacing
  left?: Spacing
  bottom?: Spacing
  right?: Spacing
  zIndex?: keyof typeof theme.zIndex | number
}

export function Box<E extends Tags = 'div'>({ as, children, onClick, innerRef, ...props }: IProps<E>) {
  const className = useStyles(props as IStyleObject)

  return React.createElement(as || 'div', {
    ref: innerRef,
    onClick,
    className,
  }, children)
}
