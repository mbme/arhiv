import * as React from 'react'
import { cssRaw } from 'typestyle'

interface IProps {
  children: string
}

export function RawCss({ children }: IProps) {
  React.useEffect(() => {
    cssRaw(children)
  }, [])

  return null
}
