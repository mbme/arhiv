import React, { PureComponent } from 'react'
import { classNames } from '../../utils'
import Icon from './Icon'
import Backdrop from './Backdrop'
import './ProgressLocker.css'

interface IState {
  visible: boolean
}

export default class ProgressLocker extends PureComponent<{}, IState> {
  state = {
    visible: false,
  }

  timer?: number

  makeVisible = () => this.setState({ visible: true })

  componentDidMount() {
    this.timer = window.setTimeout(this.makeVisible, 1000)
  }

  componentWillUnmount() {
    clearTimeout(this.timer)
  }

  render() {
    return (
      <Backdrop className={classNames('Progress-backdrop', { 'is-visible': this.state.visible })}>
        <Icon type="loader" className="Progress-spinner" />
      </Backdrop>
    )
  }
}
