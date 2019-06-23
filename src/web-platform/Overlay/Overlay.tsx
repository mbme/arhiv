import * as React from 'react'
import { Counter } from '~/utils'
import {
  StylishStyle,
  stylish,
  theme,
} from '../style'

const $container = stylish({
  backgroundColor: theme.color.bgOverlay,

  position: 'fixed',
  top: '0',
  right: '0',
  bottom: '0',
  left: '0',
  zIndex: theme.zIndex.modal,

  display: 'flex',
  justifyContent: 'center',
  alignItems: 'flex-start',
})

interface IOverlay {
  children: React.ReactNode
  onClick?(): void
  $style?: StylishStyle
}

interface IOverlayRenderer {
  show(id: number, overlay: IOverlay): void
  hide(id: number): void
}

const OverlayContext = React.createContext<IOverlayRenderer>(null as any)

interface IProps {
  children: React.ReactNode,
}

interface IState {
  overlays: ReadonlyArray<[number, IOverlay]>,
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

    const [id, overlay] = overlays[overlays.length - 1]

    const clickHandler = (e: React.MouseEvent<HTMLDivElement>) => {
      if (overlay.onClick && e.target === e.currentTarget) {
        overlay.onClick()
      }
    }

    return (
      <div
        key={id}
        className={$container.and(overlay.$style).className}
        onClick={clickHandler}
      >
        {overlay.children}
      </div>
    )
  }

  render() {
    const {
      children,
    } = this.props

    return (
      <OverlayContext.Provider value={this.renderer}>
        {children}
        {this.getOverlay()}
      </OverlayContext.Provider>
    )
  }
}

const idCounter = new Counter()

export function Overlay(props: IOverlay) {
  const renderer = React.useContext(OverlayContext)

  React.useEffect(() => {
    const id = idCounter.incAndGet()

    renderer.show(id, props)

    return () => renderer.hide(id)
  })

  return null
}
