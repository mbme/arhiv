import * as React from 'react'
import { Arhiv } from '~/arhiv'
import { AuthOverlay } from './AuthOverlay'

interface IProps {
  arhiv: Arhiv
}

interface IState {
  authorized: boolean
}

export class AuthManager extends React.PureComponent<IProps, IState> {
  state: IState = {
    authorized: this.props.arhiv.net.$authorized.currentValue,
  }

  _unsubscribe?: () => void

  componentDidMount() {
    const {
      arhiv,
    } = this.props

    this._unsubscribe = arhiv.net.$authorized.subscribe((authorized) => {
      this.setState({ authorized })
    })
  }

  componentWillUnmount() {
    if (this._unsubscribe) {
      this._unsubscribe()
    }
  }

  tryPassword = (password: string) => this.props.arhiv.net.authorize(password)

  render() {
    const {
      authorized,
    } = this.state

    if (authorized) {
      return null
    }

    return (
      <AuthOverlay submit={this.tryPassword} />
    )
  }
}
