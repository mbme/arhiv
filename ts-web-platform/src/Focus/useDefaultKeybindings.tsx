import * as React from 'react'
import {
  HotkeysResolverContext,
  IKeybinding,
} from '@v/web-utils'
import { FocusManager } from './focus-manager'

export function useDefaultKeybindings(focusManager: FocusManager) {
  const hotkeysResolver = HotkeysResolverContext.use()

  React.useEffect(() => {
    const hotkeys: IKeybinding[] = [
      {
        code: focusManager.mode === 'row' ? 'KeyL' : 'KeyJ',
        action() {
          focusManager.selectNextChild()
        },
      },
      {
        code: focusManager.mode === 'row' ? 'KeyH' : 'KeyK',
        action() {
          focusManager.selectPreviousChild()
        },
      },
      {
        code: 'Tab',
        preventDefault: true,
        action() {
          focusManager.selectNextChild()
        },
      },
      {
        code: 'Tab',
        shiftKey: true,
        preventDefault: true,
        action() {
          focusManager.selectPreviousChild()
        },
      },
      {
        code: 'Enter',
        preventDefault: true,
        action() {
          focusManager.activateSelectedChild()
        },
      },
    ]

    const unsub = focusManager.enabled$.value$.subscribe({
      next(isEnabled) {
        if (isEnabled) {
          hotkeysResolver.add(focusManager.depth, hotkeys)
        } else {
          hotkeysResolver.remove(hotkeys)
        }
      },
    })

    return () => {
      unsub()
      hotkeysResolver.remove(hotkeys)
    }
  }, [])
}
