import * as React from 'react'
import {
  style,
  classes,
} from 'typestyle'

const imageStyles = (width?: string, height?: string) => style(
  !!width && {
    width,
  },
  !!height && {
    height,
  },
)

interface IProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  width?: string
  height?: string
}

export function Image({ className, width, height, ...props }: IProps) {
  return (
    <img
      className={classes(imageStyles(width, height), className)}
      {...props}
    />
  )
}
