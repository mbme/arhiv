import * as React from 'react'
import {
  Box,
  Input,
  Row,
  Button,
  Spacer,
  clickOnEnter,
  StyleArg,
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
      py="fine"
      width="100%"
      bgColor="bg1"
      zIndex={1}
      $style={$container}
    >
      <div
        role="tab"
        tabIndex={0}
        onKeyPress={clickOnEnter}
      >
        Catalog
      </div>

      <Spacer
        flex="0"
        width="medium"
      />

      <Box
        width="11rem"
        mr="large"
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
