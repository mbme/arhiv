import * as React from 'react'
import { animation } from '../styles'
import { Icon } from '../Icon'
import { Overlay } from './Overlay'
import { styleRules } from '~/styler';

const $overlay = styleRules(
  {
    cursor: 'progress',
    alignItems: 'center',
    opacity: 0,
  },
  props => props.isVisible && {
    animation: `${animation.pulse} 3s infinite`,
  },
)

const $spinner = {
  width: '24px',
  height: '24px',
  animation: `${animation.spin} 1.5s infinite`,
}

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
    return (
      <Overlay $style={$overlay(this.state)}>
        <Icon
          type="loader"
          $style={$spinner}
        />
      </Overlay>
    )
  }
}
