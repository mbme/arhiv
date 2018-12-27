import React, { PureComponent } from 'react'
import { inject, ActionsType, StateType } from '../store'
import { Icon } from '../components'
import './Toolbar.css'

interface IProps {
  left?: React.ReactNode
  right?: React.ReactNode
  isNavVisible: boolean
  showNav: (show: boolean) => void
}
class Toolbar extends PureComponent<IProps, {}> {
  toggleNav = () => this.props.showNav(!this.props.isNavVisible)

  render() {
    return (
      <div className="Toolbar">
        <div className="Toolbar-cell is-left">
          <Icon type="menu" className="Toolbar-menu-icon" onClick={this.toggleNav} />
          {this.props.left}
        </div>
        <div className="Toolbar-cell is-right">
          {this.props.right}
        </div>
      </div>
    )
  }
}

const mapStoreToProps = (state: StateType, actions: ActionsType) => ({
  isNavVisible: state.isNavVisible,
  showNav: actions.showNav,
})

export default inject(mapStoreToProps, Toolbar)
