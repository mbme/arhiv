import * as React from 'react'
import { Row } from './Layout'
import { Icon, icons, IconType } from './Icon'
import { Example } from './Example'

export function IconExamples() {
  return (
    <Example section title="Icons">
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
    </Example>
  )
}
