import * as React from 'react'
import { mergeStyles } from '@v/web-utils'
import {
  animations,
} from '../style'
import { Icon } from '../Icon'
import { Overlay } from './Overlay'

export const ProgressLocker = React.memo(() => {
  const [visible, setVisible] = React.useState(false)

  const $style = mergeStyles([
    {
      cursor: 'progress',
      alignItems: 'center',
      opacity: 0,
    },
    visible && {
      animation: `${animations.pulse} 3s infinite`,
    },
  ])

  React.useEffect(() => {
    const timer = window.setTimeout(() => {
      setVisible(true)
    }, 1000)

    return () => clearTimeout(timer)
  }, [])

  return (
    <Overlay $style={$style}>
      <Icon
        type="loader"
        $style={{
          width: '24px',
          height: '24px',
          animation: `${animations.spin} 1.5s infinite`,
        }}
      />
    </Overlay>
  )
})
