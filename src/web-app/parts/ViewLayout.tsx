import React, { PureComponent } from 'react';
import { Backdrop } from '../components'
import { classNames } from '../../utils';
import { deauthorize } from '../utils';
import Link from './Link'
import Toaster from './Toaster'
import './Navbar.css'

type Route = 'notes' | 'theme'

interface IProps {
  selected: Route
  isNavVisible: boolean
  showNav: (show: boolean) => void
  children: JSX.Element
}

export default class ViewLayout extends PureComponent<IProps, {}> {
  logout = () => {
    deauthorize();
    window.location.reload();
  }

  render() {
    const {
      selected,
      children,
      showNav,
      isNavVisible,
    } = this.props

    const navbar = (
      <nav className="App-navbar">
        <Link
          clean
          to={{ path: '/notes' }}
          className={classNames('App-navlink', { 'is-selected': selected === 'notes' })}
        >
          Notes
        </Link>

        <Link
          clean
          to={{ path: '/theme' }}
          className={classNames('App-navlink', { 'is-selected': selected === 'theme' })}
        >
          Theme
        </Link>

        <div className="App-logout" onClick={this.logout}>
          Logout
        </div>
      </nav>
    );

    return (
      <div className="App-container">
        <div className="App-navbar-container">{navbar}</div>

        {isNavVisible && (
          <Backdrop onClick={() => showNav(false)}>
            {navbar}
          </Backdrop>
        )}

        <div className="App-view">
          {children}
        </div>

        <Toaster />
      </div>
    );
  }
}
