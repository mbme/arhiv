import * as React from 'react'
import { noop } from '@v/utils'
import { Select } from './Select'
import { Example } from './Example'

export function SelectExamples() {
  return (
    <Example section title="Select">
      <Select
        name="select"
        options={{ val1: 'val1', val2: 'val2' }}
        onChange={noop}
      />
    </Example>
  )
}
