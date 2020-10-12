import * as React from 'react'
import { noop } from '@v/utils'
import {
  HotkeysResolverContext,
  IKeybinding,
} from '@v/web-utils'
import { FocusManager } from './focus-manager'

export function useDefaultKeybindings(focusManager?: FocusManager) {
  const hotkeysResolver = HotkeysResolverContext.use()

  React.useEffect(() => {
    if (!focusManager) {
      return noop
    }

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
        code: 'Enter',
        action(e) {
          e.preventDefault()
          focusManager.activateSelectedChild()
        },
      },
    ]

    return focusManager.enabled$.value$.subscribe({
      next(isEnabled) {
        if (isEnabled) {
          hotkeysResolver.add(hotkeys)
        } else {
          hotkeysResolver.remove(hotkeys)
        }
      },
    })
  }, [focusManager, hotkeysResolver])
}
