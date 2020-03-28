import * as React from 'react'
import { noop } from '@v/utils'
import { Select } from './Select'
import { Examples } from './Examples'

const examples = {
  '': (
    <Select
      name="select"
      options={{ val1: 'val1', val2: 'val2' }}
      onChange={noop}
    />
  ),
}

export function SelectExamples() {
  return (
    <Examples title="Select" examples={examples} />
  )
}
