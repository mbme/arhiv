import * as React from 'react'
import {
  Box,
  StyleArg,
  Column,
  Link,
  Spacer,
} from '@v/web-platform'
import { ActionsRenderer } from './ActionsRenderer'
import { ActionsRegistry } from './context'

const $container: StyleArg = {
  display: 'grid',
  gridTemplateColumns: 'auto 250px',
  gridTemplateRows: '50px auto',
  gridAutoFlow: 'row',

  minHeight: '100vh',

  maxWidth: '54rem',
  minWidth: '34rem',
  mx: 'auto',
}

const $header: StyleArg = {
  bgColor: '#88c0d0',

  px: 'medium',
  py: 'small',

  display: 'flex',

  position: 'sticky',
  top: 0,
  zIndex: 1,
}

const $content: StyleArg = {
  gridRow: '2',
  pt: 'medium',
  pl: 'medium',
  height: '100%',
  bgColor: 'var(--color-bg0)',
}

const $actions: StyleArg = {
  gridRow: '2',
  pl: 'medium',
}

const $actionsBox: StyleArg = {
  position: 'sticky',
  top: '50px',
}

interface IProps {
  children: React.ReactNode
}

export function Frame({ children }: IProps) {
  return (
    <ActionsRegistry.Provider>
      <Box
        as="section"
        $style={$container}
      >
        <Box
          $style={$header}
        >
          <Link to="/" clean>
            Dashboard
          </Link>

          <Spacer
            width="1rem"
            flex="0 0 1rem"
          />

          <span>
            Stats
          </span>

          <Spacer />

          <span>
            Synced
          </span>
        </Box>

        <Box
          $style={$content}
        >
          <Column
            height="100%"
            alignX="stretch"
          >
            {children}
          </Column>
        </Box>

        <Column
          $style={$actions}
        >
          <Box $style={$actionsBox}>
            <ActionsRenderer />
          </Box>
        </Column>
      </Box>
    </ActionsRegistry.Provider>
  )
}
