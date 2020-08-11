import * as React from 'react'
import {
  Input,
  Row,
  StyleArg,
} from '@v/web-platform'

const $container: StyleArg = {
  position: 'sticky',
  top: 0,
}

interface IProps {
  filter: string
  onChange(filter: string): void
}

export function Header({ filter, onChange }: IProps) {

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
    </Row>
  )
}
