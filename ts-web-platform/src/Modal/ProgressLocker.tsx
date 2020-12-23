import * as React from 'react'
import {
  useAnimation,
  StyleArg,
} from '../core'
import { Icon } from '../Icon'
import Overlay from './Overlay'

function ProgressLocker() {
  const [visible, setVisible] = React.useState(false)

  const pulseAnimation = useAnimation('pulse')
  const spinAnimation = useAnimation('spin')
  const $styles: StyleArg[] = [
    {
      cursor: 'progress',
      alignItems: 'center',
      opacity: 0,
    },
    visible && {
      animation: `${pulseAnimation} 3s infinite`,
    },
  ]

  React.useEffect(() => {
    const timer = window.setTimeout(() => {
      setVisible(true)
    }, 1000)

    return () => clearTimeout(timer)
  }, [])

  return (
    <Overlay $styles={$styles}>
      <Icon
        type="loader"
        $style={{
          width: '24px',
          height: '24px',
          animation: `${spinAnimation} 1.5s infinite`,
        }}
      />
    </Overlay>
  )
}

export default React.memo(ProgressLocker)
