import * as React from 'react'
import { Link } from '@v/web-utils/src/router'
import { IStyleObject, useStyles } from '@v/web-utils'

const $cleanLink = {
  color: 'inherit',
  '&:hover': {
    color: 'inherit',
  },
}

interface IProps extends Omit<React.ComponentProps<typeof Link>, 'className'> {
  $style?: IStyleObject
}

export function CleanLink({ $style, ...props }: IProps) {
  const className = useStyles($cleanLink, $style)

  return  (
    <Link className={className} {...props} />
  )
}
