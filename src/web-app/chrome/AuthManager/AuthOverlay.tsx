import * as React from 'react'
import {
  style,
} from 'typestyle'
import {
  Overlay,
  Input,
  theme,
  Box,
  Image,
} from '~/web-components'

const $overlay = style({
  backgroundColor: theme.color.bg,
  paddingTop: '20vh',

  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'flex-start',
  alignItems: 'center',
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
    <Overlay className={$overlay}>
      <Box mb="medium">
        <Image
          width="150px"
          // FIXME image should have mb="medium" itself
          src="/logo.svg"
          alt="logo"
        />
      </Box>

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
