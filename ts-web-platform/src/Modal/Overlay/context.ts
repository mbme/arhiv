import {
  createContext,
} from '@v/web-utils'
import {
  StyleArg,
} from '../../core'

export interface IOverlay {
  children: React.ReactNode
  onClick?(): void
  $styles?: StyleArg[]
  $style?: StyleArg
  innerRef?: React.RefObject<HTMLDivElement>
}

export interface IOverlayRenderer {
  show(id: number, overlay: IOverlay): void
  hide(id: number): void
}

export const OverlayContext = createContext<IOverlayRenderer>()
