import * as React from 'react'
import {
  Box,
  StyleArg,
  Column,
  Link,
  Heading,
  Spacer,
  Button,
} from '@v/web-platform'
import {
  ActionsContext,
  IAction,
} from './actions'

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

const $action: StyleArg = {
  textTransform: 'uppercase',
  width: '100%',
}

interface IProps {
  children: React.ReactNode
  title: string
}

export function Frame({ children, title  }: IProps) {
  const [actions, setActions] = React.useState<IAction[]>([])

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


          <ActionsContext.Provider value={setActions}>
            {children}
          </ActionsContext.Provider>
        </Column>
      </Box>

      <Column
        $style={$actions}
      >
        {actions.map((action, i) => (
          <Button
            key={i}
            onClick={action.onClick}
            disabled={action.disabled}
            $style={$action}
          >
            {action.children}
          </Button>
        ))}
      </Column>
    </Box>
  )
}
