import React, { PureComponent } from 'react'
import { inject, AppStore } from '../store'
import './Toaster.css'

const TOAST_TIMEOUT_MS = 8000

interface IProps {
  toast?: JSX.Element
  hideToast(): void
}
class Toaster extends PureComponent<IProps> {
  toastTimeout?: number

  componentDidUpdate(prevProps: IProps) {
    if (this.props.toast && this.props.toast !== prevProps.toast) { // hide toast in few seconds
      clearTimeout(this.toastTimeout)
      this.toastTimeout = window.setTimeout(this.props.hideToast, TOAST_TIMEOUT_MS)
    }
  }

  componentWillUnmount() {
    clearTimeout(this.toastTimeout)
  }

  render() {
    return (
      <div className="Toaster-container">
        {this.props.toast}
      </div>
    )
  }
}

const mapStoreToProps = (store: AppStore) => ({
  toast: store.state.toast,
  hideToast: store.hideToast,
})

export default inject(mapStoreToProps, Toaster)
