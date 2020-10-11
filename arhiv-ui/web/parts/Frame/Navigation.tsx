import * as React from 'react'
import {
  Box,
  Column,
  Link,
} from '@v/web-platform'

export function Navigation() {
  return (
    <Column>
      <Box
        bgColor="var(--color-primary)"
        width="100%"
        display="flex"
        justifyContent="center"
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
  )
}
