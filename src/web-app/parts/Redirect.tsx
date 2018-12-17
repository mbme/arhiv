import { PureComponent } from 'react'
import { IRoute } from '../../web-router'
import { inject, IStore } from '../store'

interface IProps {
  to: IRoute
  replace: (route: IRoute) => void
}

class Redirect extends PureComponent<IProps, {}> {
  componentDidMount() {
    this.props.replace(this.props.to)
  }

  render() {
    return undefined
  }
}

const mapStoreToProps = (store: IStore) => ({
  replace: store.replace,
})

export default inject(mapStoreToProps, Redirect)
