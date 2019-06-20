import * as React from 'react'
import { Link } from '~/web-router'
import {
  stylish,
  Stylish,
} from '~/stylish'

const $cleanLink = stylish({
  color: 'inherit',
  '&:hover': {
    color: 'inherit',
  },
})

interface IProps extends Omit<React.ComponentProps<typeof Link>, 'className'> {
  $style?: Stylish
}

export const CleanLink = ({ $style, ...props }: IProps) => (
  <Link className={$cleanLink.and($style).className} {...props} />
)
