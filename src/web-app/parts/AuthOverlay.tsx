import * as React from 'react'
import {
  Overlay,
  Input,
  Box,
  Image,
  Column,
} from '~/web-platform'

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
    <Column
      as={Overlay}
      bgColor="bg0"
      pt="20vh"
    >
      <Box mb="medium">
        <Image
          width="150px"
          src="/logo.svg"
          alt="logo"
        />
      </Box>

      <Box widht="300px">
        <Input
          name="password"
          type="password"
          autoComplete="off"
          autoFocus
          value={password}
          onChange={setPassword}
          onKeyDown={onKeyDown}
        />
      </Box>
    </Column>
  )
}
