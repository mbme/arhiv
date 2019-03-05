import { PureComponent } from 'react'
import { OptionalProps } from '../../utils'
import { IRoute } from '../../web-router'
import { inject, AppStore } from '../store'

interface IProps {
  to: OptionalProps<IRoute, 'params'>
  replace(route: IRoute): void
}

class Redirect extends PureComponent<IProps> {
  componentDidMount() {
    this.props.replace({
      path: this.props.to.path,
      params: this.props.to.params || {},
    })
  }

  render() {
    return null
  }
}

const mapStoreToProps = (store: AppStore) => ({
  replace: store.replace,
})

export default inject(mapStoreToProps, Redirect)
