import { IKeybinding } from './utils'
import { createContext } from '../context'

export interface IHotkeyResolver {
  add(hotkeys: IKeybinding[]): void
  remove(hotkeys: IKeybinding[]): void
  addDocument(document: Document): void
  removeDocument(document: Document): void
}

export const HotkeysResolverContext = createContext<IHotkeyResolver>()
