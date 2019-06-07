import * as React from 'react'
import {
  style,
  classes,
} from 'typestyle'
import { Link } from '~/web-router'

const cleanLinkStyles = style({
  color: 'inherit',

  $nest: {
    '&:hover': {
      color: 'inherit',
    },
  },
})

export const CleanLink = ({ className, ...props }: React.ComponentProps<typeof Link>) => (
  <Link className={classes(cleanLinkStyles, className)} {...props} />
)
