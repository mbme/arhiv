import * as React from 'react'
import {
  Box,
  Input,
  Row,
  Button,
  Spacer,
  StyleArg,
  Link,
} from '@v/web-platform'
import { RouterContext } from '@v/web-utils'

const $container: StyleArg = {
  position: 'sticky',
  top: 0,
}

export function Header() {
  const router = RouterContext.use()

  function onChange(newFilter: string) {
    console.error(newFilter)
  }

  return (
    <Row
      as="nav"
      alignX="center"
      boxShadow="default"
      p="fine"
      width="100%"
      bgColor="bg1"
      zIndex={1}
      $style={$container}
    >
      <Link to={{ path: '/' }}>
        Catalog
      </Link>

      <Spacer flex="1" />

      <Box
        minWidth="11rem"
        maxWidth="22rem"
        mx="large"
      >
        <Input
          light
          name="filter"
          placeholder="Filter documents"
          value="filter"
          onChange={onChange}
          onClear={() => onChange('')}
          onKeyDown={(e) => {
            if (e.key === 'Escape') {
              onChange('')
            }
          }}
        />
      </Box>

      <Spacer flex="1" />

      <Button
        variant="primary"
        onClick={() => router.push({ path: '/new' })}
      >
        Add
      </Button>
    </Row>
  )
}
