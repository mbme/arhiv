import * as React from 'react'
import { noop } from '@v/utils'
import { Select } from './Select'

export const examples = {
  '': (
    <Select
      name="select"
      options={{ val1: 'val1', val2: 'val2' }}
      onChange={noop}
    />
  ),
}
