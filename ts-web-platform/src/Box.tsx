import * as React from 'react'
import { useStyles, IStyleObject, StyleArg } from '@v/web-utils'
import { theme } from './style'
import { Tags, Spacing } from './types'

export interface IProps<E extends Tags> {
  as?: E
  children?: React.ReactNode
  onClick?(e: React.MouseEvent<HTMLElementTagNameMap[E]>): void
  innerRef?: React.RefObject<HTMLElementTagNameMap[E]>

  $style?: StyleArg

  relative?: boolean
  absolute?: boolean
  hidden?: boolean

  display?: string
  flex?: string
  flexDirection?: string
  alignItems?: string
  justifyContent?: string

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

  color?: keyof typeof theme.color | string
  bgColor?: keyof typeof theme.color | string
  backgroundColor?: keyof typeof theme.color | string

  top?: Spacing
  left?: Spacing
  bottom?: Spacing
  right?: Spacing
  zIndex?: keyof typeof theme.zIndex | number
}

export function Box<E extends Tags = 'div'>({ as, children, onClick, innerRef, $style, ...props }: IProps<E>) {
  const className = useStyles($style, props as IStyleObject)

  return React.createElement(as || 'div', {
    ref: innerRef,
    onClick,
    className,
  }, children)
}
