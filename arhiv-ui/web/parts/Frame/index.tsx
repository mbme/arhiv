import * as React from 'react'
import {
  Box,
  StyleArg,
  FocusRegion,
  Column,
} from '@v/web-platform'
import { Navigation } from './Navigation'
import { Actions } from './Actions'

const $container: StyleArg = {
  display: 'grid',
  gridTemplateColumns: 'minmax(100px, 1fr) 3fr minmax(100px, 1fr)',
  gridAutoFlow: 'column',
  gridGap: '0.8rem',

  height: '100vh',
  overflowY: 'hidden',
  p: 'fine',

  maxWidth: '80rem',
  minWidth: '44rem',
  mx: 'auto',
}

const $content: StyleArg = {
  height: '100%',
  overflowY: 'auto',
}

interface IProps {
  children: React.ReactNode
  actions: React.ReactNode
  title: string
}

export function Frame({ children, actions, title  }: IProps) {
  return (
    <FocusRegion
      name="Frame"
      mode="row"
    >
      <Box
        as="section"
        $style={$container}
      >
        <FocusRegion
          name="Navigation"
          mode="column"
          highlight
          title="Apps"
        >
          <Navigation />
        </FocusRegion>

        <FocusRegion
          name="Content"
          mode="column"
          highlight
          title={title}
          $style={$content}
          autoFocus
        >
          <Column
            height="100%"
            overflowY="auto"
            alignX="stretch"
          >
            {children}
          </Column>
        </FocusRegion>

        <FocusRegion
          name="Actions"
          mode="column"
          highlight
          title="Actions"
        >
          <Actions actions={actions} />
        </FocusRegion>
      </Box>
    </FocusRegion>
  )
}
