import * as React from 'react'
import { Example } from '../Example'
import { Link } from './Link'

export function LinkExamples() {
  return (
    <Example section title="Links">
      <Link to="/">Regular link</Link>

      <br />

      <Link to="/" clean>Clean link</Link>
    </Example>
  )
}
