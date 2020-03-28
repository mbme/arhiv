import * as React from 'react'
import {
  animations,
} from '../style'
import { Icon } from '../Icon'
import { Overlay } from './Overlay'

const $overlay = stylish(
  {
    cursor: 'progress',
    alignItems: 'center',
    opacity: 0,
  },
  props => props.visible && {
    animation: `${animations.pulse} 3s infinite`,
  },
)

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
      <Overlay $style={$overlay.with(this.state)}>
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
  }
}
