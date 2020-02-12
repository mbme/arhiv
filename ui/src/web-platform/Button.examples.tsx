import * as React from 'react'
import { Button } from './Button'

export const examples = {
  'Primary': (
    <Button variant="primary">Primary Button</Button>
  ),
  'Primary disabled': (
    <Button variant="primary" disabled>Primary Button</Button>
  ),

  'Secondary': (
    <Button>Button</Button>
  ),
  'Secondary disabled': (
    <Button disabled>Button</Button>
  ),

  'Link': (
    <Button variant="link">Button</Button>
  ),
  'Link disabled': (
    <Button variant="link" disabled>Button</Button>
  ),
}
