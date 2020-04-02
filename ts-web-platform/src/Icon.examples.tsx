import * as React from 'react'
import { Row } from './Layout'
import { Icon, icons, IconType } from './Icon'
import { Examples } from './Examples'

const examples = {
  '': (
    <Row>
      {Object.keys(icons).map(iconType => (
        <Icon
          key={iconType}
          $styles={[{
            margin: '1rem',
            flex: '1 1 auto',
          }]}
          type={iconType as IconType}
          title={iconType}
        />
      ))}
    </Row>
  )
}

export function IconExamples() {
  return (
    <Examples title="Icons" examples={examples} />
  )
}
