import * as React from 'react'
import {
  Tags,
  StyleProps,
  StyleArg,
  useStyles,
} from './core'

type PassThroughProps =
  'relative'
  | 'absolute'
  | 'hidden'
  | 'display'
  | 'flex'
  | 'flexDirection'
  | 'alignItems'
  | 'justifyContent'

  | 'm'
  | 'mx'
  | 'my'
  | 'mt'
  | 'mr'
  | 'mb'
  | 'ml'

  | 'p'
  | 'px'
  | 'py'
  | 'pt'
  | 'pr'
  | 'pb'
  | 'pl'

  | 'top'
  | 'right'
  | 'bottom'
  | 'left'
  | 'zIndex'

  | 'width'
  | 'height'

  | 'color'
  | 'bgColor'
  | 'backgroundColor'

  | 'fromSm'
  | 'fromMd'
  | 'fromLg'

export interface IProps<E extends Tags> extends Pick<StyleProps, PassThroughProps> {
  as?: E
  children?: React.ReactNode
  onClick?(e: React.MouseEvent<HTMLElementTagNameMap[E]>): void
  innerRef?: React.RefObject<HTMLElementTagNameMap[E]>
  $styles?: StyleArg[]
}

export function Box<E extends Tags = 'div'>({ as, children, onClick, innerRef, $styles = [], ...props }: IProps<E>) {
  const className = useStyles(props, ...$styles)

  return React.createElement(as || 'div', {
    ref: innerRef,
    onClick,
    className,
  }, children)
}
