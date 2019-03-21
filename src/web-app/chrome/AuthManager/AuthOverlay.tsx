import * as React from 'react'
import {
  style,
} from 'typestyle'
import {
  Overlay,
  Input,
  theme,
} from '~/web-components'

const overlayStyles = style({
  backgroundColor: theme.color.bg,
  paddingTop: '20vh',

  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'flex-start',
  alignItems: 'center',
})

const logoStyles = style({
  width: '150px',
  marginBottom: theme.spacing.medium,
})

interface IProps {
  submit(password: string): void
}

export function AuthOverlay({ submit }: IProps) {
  const [password, setPassword] = React.useState('')

  function onKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Enter') {
      submit(password)
    }
  }

  return (
    <Overlay className={overlayStyles}>
      <img alt="logo" src="/logo.svg" className={logoStyles} />
      <Input
        className={style({ width: '300px' })}
        name="password"
        type="password"
        autoFocus
        value={password}
        onChange={setPassword}
        onKeyDown={onKeyDown}
      />
    </Overlay>
  )
}
