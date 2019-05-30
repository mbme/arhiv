import * as React from 'react'
import { IsodbWebClient } from '~/isodb-web-client'
import { AuthOverlay } from './AuthOverlay'

interface IProps {
  client: IsodbWebClient
}

interface IState {
  authorized: boolean
}

export class AuthManager extends React.PureComponent<IProps, IState> {
  state: IState = {
    authorized: this.props.client.isAuthorized(),
  }

  onAuthorized = (authorized: boolean) => {
    this.setState({ authorized })
  }

  componentDidMount() {
    this.props.client.events.on('authorized', this.onAuthorized)
  }

  componentWillUnmount() {
    this.props.client.events.off('authorized', this.onAuthorized)
  }

  tryPassword = (password: string) => this.props.client.authorize(password)

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
