import * as React from 'react'
import {
  Box,
  Column,
  StyleArg,
  Link,
} from '@v/web-platform'

const $container: StyleArg = {
  display: 'grid',
  gridTemplateColumns: '10rem auto 10rem 1fr',
  gridAutoFlow: 'column',
  height: '100vh',
  overflowY: 'hidden',
}

interface IProps {
  children: React.ReactNode,
  actions: React.ReactNode,
}

export function Frame({ children, actions }: IProps) {
  return (
    <Box
      as="section"
      $style={$container}
    >
      <Column>
        <Box
          bgColor="var(--color-primary)"
          width="100%"
        >
          <Link to={{ path: '/' }}>
            Notes
          </Link>
        </Box>
        <Link to={{ path: '/' }}>
          Contacts
        </Link>
        <Link to={{ path: '/' }}>
          Movies
        </Link>
      </Column>

      <Box
        px="medium"
        pt="medium"
        width="40rem"
        overflowY="auto"
        borderLeft="default"
        borderRight="default"
      >
        {children}
      </Box>

      <Column>
        {actions}
      </Column>
    </Box>
  )
}
