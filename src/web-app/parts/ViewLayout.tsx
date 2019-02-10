import React, { PureComponent } from 'react'
import { inject, AppStore } from '../store'
import { Backdrop } from '../components'
import { classNames } from '../../utils'
import Link from './Link'
import Toaster from './Toaster'
import './ViewLayout.css'
import { IRoute } from '../../web-router'

interface IProps {
  children: React.ReactNode

  route: IRoute
  isNavVisible: boolean
  showNav(show: boolean): void
  deauthorize(): void
}

class ViewLayout extends PureComponent<IProps> {
  logout = async () => {
    this.props.deauthorize()
    window.location.reload()
  }

  hideNav = () => {
    this.props.showNav(false)
  }

  render() {
    const {
      route,
      children,
      isNavVisible,
    } = this.props

    const isNotes = route.path.startsWith('/notes')
    const isTheme = route.path.startsWith('/theme')

    const navbar = (
      <nav className="App-navbar">
        <Link
          clean
          to={{ path: '/notes' }}
          className={classNames('App-navlink', { 'is-selected': isNotes })}
        >
          Notes
        </Link>

        <Link
          clean
          to={{ path: '/theme' }}
          className={classNames('App-navlink', { 'is-selected': isTheme })}
        >
          Theme
        </Link>

        <div className="App-logout" onClick={this.logout}>
          Logout
        </div>
      </nav>
    )

    return (
      <div className="App-container">
        <div className="App-navbar-container">{navbar}</div>

        {isNavVisible && (
          <Backdrop onClick={this.hideNav}>
            {navbar}
          </Backdrop>
        )}

        <div className="App-view">
          {children}
        </div>

        <Toaster />
      </div>
    )
  }
}

const mapStoreToProps = (store: AppStore) => ({
  route: store.state.route,
  isNavVisible: store.state.isNavVisible,
  showNav: store.showNav,
  deauthorize: store.deauthorize,
})

export default inject(mapStoreToProps, ViewLayout)
