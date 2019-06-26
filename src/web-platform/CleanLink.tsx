import * as React from 'react'
import { Link } from '~/web-router'
import {
  stylish,
  $Style,
} from './style'

const $cleanLink = stylish({
  color: 'inherit',
  '&:hover': {
    color: 'inherit',
  },
})

interface IProps extends Omit<React.ComponentProps<typeof Link>, 'className'> {
  $style?: $Style
}

export const CleanLink = ({ $style, ...props }: IProps) => (
  <Link className={$cleanLink.and($style).className} {...props} />
)
