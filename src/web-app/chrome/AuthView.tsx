import React, { PureComponent } from 'react'
import { inject, ActionsType, IStoreState } from '../store'
import { Backdrop, Input } from '../components'
import './AuthView.css'

interface IProps {
  authorize(password: string): Promise<void>
}

interface IStoreState {
  password: string
}

class AuthView extends PureComponent<IProps, IStoreState> {
  state = {
    password: '',
  }

  onPasswordChange = (password: string) => this.setState({ password })

  onKeyDown = async (e: React.KeyboardEvent) => {
    // TODO add error message
    if (e.key === 'Enter') {
      await this.props.authorize(this.state.password)
    }
  }

  render() {
    return (
      <Backdrop className="Auth-backdrop">
        <img alt="logo" src="/logo.svg" className="Auth-logo" />
        <Input
          className="Auth-input"
          name="password"
          type="password"
          autoFocus
          value={this.state.password}
          onChange={this.onPasswordChange}
          onKeyDown={this.onKeyDown}
        />
      </Backdrop>
    )
  }
}

const mapStoreToProps = (_: IStoreState, actions: ActionsType) => ({
  authorize: actions.authorize,
})

export default inject(mapStoreToProps, AuthView)
