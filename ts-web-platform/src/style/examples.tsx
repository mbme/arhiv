import * as React from 'react'
import { theme } from './theme'

export const examples = {
  'Colors': (
    <div>
      {Object.entries(theme.color).map(([name, value]) => (
        <div
          key={name}
          className={stylish({
            height: '3rem',
            backgroundColor: value,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }).className}
        >
          {name}
        </div>
      ))}
    </div>
  ),
}
