import React, { PureComponent } from 'react'
import { classNames } from '../../utils'
import { IRoute } from '../../web-router'
import { inject, IStore } from '../store'
import './Link.css'

interface ILinkRoute {
  path: string
  params?: { [key: string]: string }
}

interface IProps {
  className?: string
  clean?: boolean
  children: React.ReactNode
  to: ILinkRoute
  push: (route: IRoute) => void
}
class Link extends PureComponent<IProps, {}> {
  onClick = () => {
    this.props.push({
      path: this.props.to.path,
      params: this.props.to.params || {},
    })
  }

  render() {
    const { className, children, clean } = this.props

    return (
      <div
        className={classNames('Link', { 'is-clean': clean }, className)}
        role="link"
        tabIndex={0}
        onClick={this.onClick}
      >
        {children}
      </div>
    )
  }
}

const mapStoreToProps = (store: IStore) => ({
  push: store.push,
})

export default inject(mapStoreToProps, Link)
