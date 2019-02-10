import React, { PureComponent } from 'react'
import { classNames, OptionalProps } from '../../utils'
import { IRoute } from '../../web-router'
import { inject, AppStore } from '../store'
import './Link.css'

interface IProps {
  className?: string
  clean?: boolean
  children: React.ReactNode
  to: OptionalProps<IRoute, 'params'>
  push(route: IRoute): void
}
class Link extends PureComponent<IProps> {
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

const mapStoreToProps = (store: AppStore) => ({
  push: store.push,
})

export default inject(mapStoreToProps, Link)
