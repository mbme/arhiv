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

  | 'gridTemplateColumns'
  | 'gridTemplateRows'
  | 'gridGap'

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
  | 'minHeight'
  | 'maxHeight'
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
  | 'borderColor'
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
  dangerouslySetInnerHTML?: {
    __html: string;
  }
  tabIndex?: string
  $style?: StyleArg
  $styles?: StyleArg[]
}

export const Box = React.forwardRef(
  function Box<E extends Tags = 'div'>(props: IProps<E>, ref: React.Ref<HTMLElementTagNameMap[E]>) {
    const {
      as,
      children,
      onClick,
      dangerouslySetInnerHTML,
      tabIndex,
      $style,
      $styles = [],
      ...styleProps
    } = props

    const className = useStyles(styleProps, $style, ...$styles)

    return React.createElement(as || 'div', {
      ref,
      onClick,
      className,
      dangerouslySetInnerHTML,
      tabIndex,
    }, children)
  } ,
)
