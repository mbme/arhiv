import * as React from 'react'
import { style } from 'typestyle'
import { animation } from '../styles'
import { Icon } from '../Icon'
import { Overlay } from './Overlay'

const backdropStyles = (isVisible?: boolean) => style(
  {
    cursor: 'progress',
    alignItems: 'center',
    opacity: 0,
  },
  isVisible && {
    animation: `${animation.pulse} 3s infinite`,
  },
)

const spinnerStyles = style({
  width: '24px',
  height: '24px',
  animation: `${animation.spin} 1.5s infinite`,
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
