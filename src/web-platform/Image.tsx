import * as React from 'react'
import {
  stylish,
  $Style,
} from './style'

const $image = stylish(
  props => props.width && { width: props.width as string },
  props => props.height && { height: props.height as string },
)

interface IProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  width?: string
  height?: string
  $style?: $Style
}

export function Image({ width, height, $style, ...props }: IProps) {
  return (
    <img
      {...props}
      className={$image.with({ width, height }).and($style).className}
    />
  )
}
