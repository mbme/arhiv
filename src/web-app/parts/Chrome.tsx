import * as React from 'react'
import {
  stylish,
  Box,
  Link,
  theme,
} from '~/web-platform'

interface IProps {
  selected: 'workspace' | 'library'
  children: React.ReactNode
}

const $navlink = stylish(
  {
    display: 'inline-block',
    mx: 'large',
  },
  props => props.isSelected && {
    color: 'primary',
  },
)

export function Chrome({ selected, children }: IProps) {
  return (
    <Box
      height="100%"
      pt="50px" // for header
      position="relative"
    >
      <Box
        as="nav"
        boxShadow={theme.boxShadow}
        py="small"
        position="absolute"
        top="0"
        width="100%"
      >
        <Link
          to={{ path: '/' }}
          className={$navlink.with({ isSelected: selected === 'workspace' }).className}
        >
          Workspace
        </Link>

        <Link
          to={{ path: '/library' }}
          className={$navlink.with({ isSelected: selected === 'library' }).className}
        >
          Library
        </Link>
      </Box>

      <Box
        height="100%"
        overflow="auto"
      >
        {children}
      </Box>
    </Box>
  )
}
