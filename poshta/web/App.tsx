import * as React from 'react'
import { Box, Modal } from '@v/web-platform'
import { useCell, useHotkeys } from '@v/web-utils'
import { PoshtaStore } from './poshta-store'
import { MessageShort } from './MessageShort'
import { MessageFull } from './MessageFull'

interface IProps {
  store: PoshtaStore,
}

export function App({ store }: IProps) {
  const [state] = useCell(store.state$)
  const modalRef = React.useRef<HTMLDivElement>(null)

  const showModal = !!state.selected

  const hotkeys = React.useMemo(() => {
    if (showModal) {
      return [
        {
          code: 'KeyJ',
          action() {
            modalRef.current?.scrollBy({ top: 100, behavior: 'smooth' })
          },
        },
        {
          code: 'KeyK',
          action() {
            modalRef.current?.scrollBy({ top: -100, behavior: 'smooth' })
          },
        },
        {
          code: 'KeyG',
          shiftKey: true,
          action() {
            modalRef.current?.scrollBy({ top: modalRef.current.scrollHeight, behavior: 'smooth' })
          },
        },
      ]
    }

    return [
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
    ]
  }, [store, showModal])

  useHotkeys(0, hotkeys)

  const items = state.messages.map((message, index) => (
    <MessageShort
      key={message.id}
      message={message}
      focused={index === state.focusedIndex}
    />
  ))

  return (
    <Box
      width="50rem"
      m="0 auto"
      p="large"
    >
      {items}

      {showModal && (
        <Modal
          onCancel={() => store.select(undefined)}
          ref={modalRef}
        >
          <Box
            maxWidth="50rem"
            minWidth="40rem"
          >
            <MessageFull message={state.selected!} />
          </Box>
        </Modal>
      )}
    </Box>
  )
}
