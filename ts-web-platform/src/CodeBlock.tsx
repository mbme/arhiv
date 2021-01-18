import * as React from 'react'

interface IProps {
  children?: React.ReactNode
}
export function CodeBlock({ children }: IProps) {
  return (
    <pre>
      <code>
        {children}
      </code>
    </pre>
  )
}
