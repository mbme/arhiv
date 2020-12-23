import {
  StyleArg,
} from '../../core'
import { createRegistry } from '../../utils'

export interface IOverlay {
  children: React.ReactNode
  onClick?(): void
  $styles?: StyleArg[]
  $style?: StyleArg
  innerRef?: React.Ref<HTMLDivElement>
}

export const OverlayRegistry = createRegistry<IOverlay>()
