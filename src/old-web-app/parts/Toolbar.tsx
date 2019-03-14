import React, { PureComponent } from 'react'
import './Toolbar.css'

interface IProps {
  left?: React.ReactNode
  right?: React.ReactNode
  isNavVisible: boolean
  showNav(show: boolean): void
}
class Toolbar extends PureComponent<IProps> {
  render() {
    return (
      <div className="Toolbar">
        <div className="Toolbar-cell is-left">
          {this.props.left}
        </div>
        <div className="Toolbar-cell is-right">
          {this.props.right}
        </div>
      </div>
    )
  }
}
