import * as React from 'react'
import {
  SimpleLocation,
  getUrl,
} from './web-router'
import { useRouter } from './Router'

interface IProps {
  to: SimpleLocation
  newTab?: boolean
  className?: string
  children: React.ReactNode
}

export function Link({ to, newTab, className, children }: IProps) {
  const router = useRouter()

  const url = getUrl(to)

  const onClick = (e: React.MouseEvent<HTMLAnchorElement>) => {
    router.push(to)
    e.preventDefault()
  }

  return (
    <a
      href={url}
      onClick={onClick}
      target={newTab ? '_blank' : undefined}
      rel={newTab ? 'noopener' : undefined}
      className={className}
    >
      {children}
    </a>
  )
}
