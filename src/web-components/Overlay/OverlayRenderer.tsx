import * as React from 'react'

interface IOverlayRenderer {
  show(id: number, overlay: React.ReactNode): void
  hide(id: number): void
}

export const OverlayContext = React.createContext<IOverlayRenderer>(null as any)

interface IProps {
  children: React.ReactNode,
}

interface IState {
  overlays: ReadonlyArray<[number, React.ReactNode]>,
}

export class OverlayRenderer extends React.PureComponent<IProps, IState> {
  state: IState = {
    overlays: [],
  }

  renderer: IOverlayRenderer = {
    show: (id, overlay) => {
      this.setState(state => ({
        overlays: [...state.overlays, [id, overlay]],
      }))
    },

    hide: (id) => {
      this.setState(state => ({
        overlays: state.overlays.filter(item => item[0] !== id),
      }))
    },
  }

  getOverlay() {
    const {
      overlays,
    } = this.state

    if (!overlays.length) {
      return null
    }

    const [, overlay] = overlays[overlays.length - 1]

    return overlay
  }

  render() {
    const {
      children,
    } = this.props

    return (
      <OverlayContext.Provider value={this.renderer}>
        {this.getOverlay()}
        {children}
      </OverlayContext.Provider>
    )
  }
}
