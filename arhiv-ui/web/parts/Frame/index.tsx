import * as React from 'react'
import {
  Box,
  StyleArg,
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
    <Box
      as="section"
      $style={$container}
    >
      <Navigation />

      <Box
        $style={$content}
      >
        <Column
          height="100%"
          overflowY="auto"
          alignX="stretch"
        >
          <Box mb="medium">
            {title}
          </Box>

          {children}
        </Column>
      </Box>

      <Actions actions={actions} />
    </Box>
  )
}
