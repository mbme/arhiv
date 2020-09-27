import * as React from 'react'
import {
  Tags,
  StyleProps,
  StyleArg,
  useStyles,
} from './core'

type PassThroughProps =
  | 'fontSize'
  | 'fontFamily'
  | 'fontWeight'
  | 'uppercase'
  | 'bold'
  | 'letterSpacing'
  | 'whiteSpace'

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
  mono?: boolean
  $styles?: StyleArg[]
}

export const Text = React.forwardRef(
  function Text<E extends Tags = 'div'>(props: IProps<E>, ref: React.Ref<HTMLElementTagNameMap[E]>) {
    const {
      as,
      children,
      onClick,
      mono,
      $styles = [],
      ...otherProps
    } = props

    const className = useStyles(otherProps, mono && { fontFamily: 'var(--font-family-mono)' }, ...$styles)

    return React.createElement(as || 'div', {
      ref,
      onClick,
      className,
    }, children)
  }
)
