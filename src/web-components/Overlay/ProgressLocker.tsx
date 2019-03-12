import * as React from 'react'
import {
  style,
  keyframes,
} from 'typestyle'
import { Icon } from '../Icon'
import { Overlay } from './Overlay'

const pulseAnimation = keyframes({
  '0%': {
    opacity: 0.7,
  },

  '50%': {
    opacity: 1,
  },

  '100%': {
    opacity: 0.7,
  },
})

const backdropStyles = (isVisible?: boolean) => style(
  {
    cursor: 'progress',
    alignItems: 'center',
    opacity: 0,
  },
  isVisible && {
    animation: `${pulseAnimation} 3s infinite`,
  },
)

const spinningAnimation = keyframes({
  from: {
    transform: 'rotate(0deg)',
  },

  to: {
    transform: 'rotate(359deg)',
  },
})

const spinnerStyles = style({
  width: '24px',
  height: '24px',
  animation: `${spinningAnimation} 1.5s infinite`,
})

interface IState {
  visible: boolean
}

export class ProgressLocker extends React.PureComponent<{}, IState> {
  state = {
    visible: false,
  }

  timer?: number

  componentDidMount() {
    this.timer = window.setTimeout(() => {
      this.setState({ visible: true })
    }, 1000)
  }

  componentWillUnmount() {
    clearTimeout(this.timer)
  }

  render() {
    const {
      visible,
    } = this.state

    return (
      <Overlay
        className={backdropStyles(visible)}
      >
        <Icon type="loader" className={spinnerStyles} />
      </Overlay>
    )
  }
}
