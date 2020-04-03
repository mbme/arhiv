import * as React from 'react'
import { noop } from '@v/utils'
import { Toggle } from './Toggle'
import { Example } from './Example'

export function ToggleExamples() {
  const [checked, setChecked] = React.useState(true)

  return (
    <Example section title="Toggle">
      <Example title="Toggle">
        <Toggle
          name="toggle"
          onChange={setChecked}
          checked={checked}
        />
      </Example>

      <Example title="Toggle disabled">
        <Toggle
          name="toggle"
          onChange={noop}
          disabled
        />
      </Example>
    </Example>
  )
}
