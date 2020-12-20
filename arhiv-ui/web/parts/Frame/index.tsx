import * as React from 'react'
import {
  Box,
  StyleArg,
  Column,
  Link,
  Heading,
  Spacer,
} from '@v/web-platform'

const $container: StyleArg = {
  display: 'grid',
  gridTemplateColumns: 'auto 250px',
  gridTemplateRows: '50px auto',
  gridAutoFlow: 'row',

  height: '100vh',
  overflowY: 'hidden',

  maxWidth: '54rem',
  minWidth: '34rem',
  mx: 'auto',
}

const $header: StyleArg = {
  bgColor: '#88c0d0',

  px: 'medium',
  py: 'small',

  display: 'flex',
}

const $content: StyleArg = {
  gridRow: '2',
  pt: 'medium',
  pl: 'medium',
  height: '100%',
  overflowY: 'auto',
  bgColor: 'var(--color-bg0)',
}

const $actions: StyleArg = {
  gridRow: '2',
  pl: 'medium',
  '&>*': {
    mb: 'small',
  },
}

interface IProps {
  children: React.ReactNode
  actions?: React.ReactNode
  title: string
}

export function Frame({ children, actions = null, title  }: IProps) {
  return (
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
          overflowY="auto"
          alignX="stretch"
        >
          <Heading
            fontSize="medium"
            uppercase
            color="var(--color-secondary)"
          >
            {title}
          </Heading>
          {children}
        </Column>
      </Box>

      <Column
        $style={$actions}
      >
        {actions}
      </Column>
    </Box>
  )
}
