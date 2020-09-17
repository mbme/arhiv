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
  | 'maxWidth'
  | 'minWidth'
  | 'height'
  | 'overflowX'
  | 'overflowY'
  | 'overflow'

  | 'color'
  | 'bgColor'
  | 'backgroundColor'

  | 'cursor'
  | 'boxShadow'

  | 'border'
  | 'borderRadius'
  | 'borderTop'
  | 'borderRight'
  | 'borderBottom'
  | 'borderLeft'

  | 'fromSm'
  | 'fromMd'
  | 'fromLg'

export interface IProps<E extends Tags> extends Pick<StyleProps, PassThroughProps> {
  as?: E
  children?: React.ReactNode
  onClick?(e: React.MouseEvent<HTMLElementTagNameMap[E]>): void
  innerRef?: React.Ref<HTMLElementTagNameMap[E]>
  dangerouslySetInnerHTML?: {
    __html: string;
  }
  $style?: StyleArg
  $styles?: StyleArg[]
}

export function Box<E extends Tags = 'div'>(props: IProps<E>) {
  const {
    as,
    children,
    onClick,
    innerRef,
    dangerouslySetInnerHTML,
    $style,
    $styles = [],
    ...styleProps
  } = props

  const className = useStyles(styleProps, $style, ...$styles)

  return React.createElement(as || 'div', {
    ref: innerRef,
    onClick,
    className,
    dangerouslySetInnerHTML,
  }, children)
}
