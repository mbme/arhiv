import * as React from 'react'
import { Box, Modal } from '@v/web-platform'
import { noop } from '@v/utils'
import { createKeybindingsHandler, useStore } from '@v/web-utils'
import { PoshtaStore } from './poshta-store'
import { MessageShort } from './MessageShort'
import { MessageFull } from './MessageFull'

function useKeybindings(store: PoshtaStore, use: boolean) {
  React.useEffect(() => {
    if (!use) {
      return noop
    }

    const handler = createKeybindingsHandler(
      {
        code: 'KeyJ',
        action() {
          store.focusNext()
        },
      },
      {
        code: 'KeyK',
        action() {
          store.focusPrev()
        },
      },
      {
        code: 'Enter',
        action() {
          store.selectFocused()
        },
      },
      {
        code: 'KeyL',
        action() {
          store.selectFocused()
        },
      },
    )

    document.addEventListener('keydown', handler)

    return () => {
      document.removeEventListener('keydown', handler)
    }
  }, [use])
}

interface IProps {
  store: PoshtaStore,
}

export function App({ store }: IProps) {
  const [state] = useStore(store)

  useKeybindings(store, !state.selected)

  const items = state.messages.map((message, index) => (
    <MessageShort
      key={message.id}
      message={message}
      focused={index === state.focusedIndex}
    />
  ))

  return (
    <Box
      maxWidth="50rem"
      m="0 auto"
      p="large"
    >
      {items}

      {state.selected && (
        <Modal onCancel={() => store.select(undefined)}>
          <Box maxWidth="50rem">
            <MessageFull
              message={state.selected}
            />
          </Box>
        </Modal>
      )}
    </Box>
  )
}
