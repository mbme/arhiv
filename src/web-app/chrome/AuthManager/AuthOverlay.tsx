import * as React from 'react'
import {
  Overlay,
  Input,
  theme,
  Box,
  Image,
  stylish,
} from '~/web-platform'

const $overlay = stylish({
  backgroundColor: theme.color.bg,
  paddingTop: '20vh',

  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'flex-start',
  alignItems: 'center',
})

const $input = stylish({
  width: '300px',
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
    <Overlay $style={$overlay}>
      <Box mb="medium">
        <Image
          width="150px"
          src="/logo.svg"
          alt="logo"
        />
      </Box>

      <Input
        $style={$input}
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
