import * as React from 'react'
import { Row } from './Layout'
import { Icon, icons, IconType } from './Icon'

export const examples = {
  '': (
    <Row>
      {Object.keys(icons).map(iconType => (
        <Icon
          key={iconType}
          $style={{
            margin: '1rem',
            flex: '1 1 auto',
          }}
          type={iconType as IconType}
          title={iconType}
        />
      ))}
    </Row>
  ),
}
