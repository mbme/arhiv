import * as React from 'react'
import {
  useStyles,
  StyleArg,
} from './core'

interface IProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  width?: string
  height?: string
  $styles?: StyleArg[]
  $style?: StyleArg
}

export function Image({ width, height, $style, $styles = [], ...props }: IProps) {
  const className = useStyles(
    width && { width },
    height && { height },
    $style,
    ...$styles,
  )

  return (
    <img
      alt=""
      {...props}
      className={className}
    />
  )
}
