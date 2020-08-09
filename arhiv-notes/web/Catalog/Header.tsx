import * as React from 'react'
import {
  Input,
  Row,
  Button,
  Spacer,
  StyleArg,
} from '@v/web-platform'
import { RouterContext } from '@v/web-utils'

const $container: StyleArg = {
  position: 'sticky',
  top: 0,
}

interface IProps {
  filter: string
  onChange(filter: string): void
}

export function Header({ filter, onChange }: IProps) {
  const router = RouterContext.use()

  return (
    <Row
      as="nav"
      alignX="center"
      p="fine"
      width="100%"
      zIndex={1}
      $style={$container}
      bgColor="var(--color-bg0)"
    >
      <Input
        name="filter"
        placeholder="Filter documents"
        value={filter}
        onChange={onChange}
        onClear={() => onChange('')}
      />

      <Spacer width="xlarge" />

      <Button
        variant="primary"
        onClick={() => router.push({ path: '/new' })}
      >
        Add
      </Button>
    </Row>
  )
}
