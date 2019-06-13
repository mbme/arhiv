import * as React from 'react'
import {
  stylish,
  Stylish,
} from '~/stylish'

const $image = stylish(
  props => props.width && ({ width: props.width }),
  props => props.height && ({ height: props.height }),
)

interface IProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  width?: string
  height?: string
  $style?: Stylish
}

export function Image({ width, height, $style, ...props }: IProps) {
  return (
    <img
      {...props}
      className={$image.with({ width, height }).and($style).className}
    />
  )
}
