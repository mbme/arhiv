import * as React from 'react'

type RenderOverlay = (overlay: React.ReactNode) => void

export const OverlayContext = React.createContext<RenderOverlay>(null as any)

interface IProps {
  children: React.ReactNode,
}

interface IState {
  overlay: React.ReactNode,
}

export class OverlayRenderer extends React.PureComponent<IProps, IState> {
  state: IState = {
    overlay: null,
  }

  renderOverlay: RenderOverlay = (overlay) => {
    this.setState({ overlay })
  }

  render() {
    const {
      children,
    } = this.props

    const {
      overlay,
    } = this.state

    return (
      <OverlayContext.Provider value={this.renderOverlay}>
        {overlay}
        {children}
      </OverlayContext.Provider>
    )
  }
}
