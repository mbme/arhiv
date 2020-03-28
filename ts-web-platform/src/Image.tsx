import * as React from 'react'
import { IStyleObject, useStyles } from '@v/web-utils'

interface IProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  width?: string
  height?: string
  $style?: IStyleObject
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
