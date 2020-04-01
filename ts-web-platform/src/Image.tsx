import * as React from 'react'
import { useStyles, StyleArg } from '@v/web-utils'

interface IProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  width?: string
  height?: string
  $style?: StyleArg
}

export function Image({ width, height, $style, ...props }: IProps) {
  const className = useStyles(
    width && { width },
    height && { height },
    $style,
  )

  return (
    <img
      alt=""
      {...props}
      className={className}
    />
  )
}
