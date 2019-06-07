import * as React from 'react'
import {
  Location,
  useRouter,
} from './Router'

interface IProps {
  to: Location
  newTab?: boolean
  className?: string
  children: React.ReactNode
}

export function Link({ to, newTab, className, children }: IProps) {
  const router = useRouter()

  const url = router.getUrl(to)

  const onClick = (e: React.MouseEvent<HTMLAnchorElement>) => {
    router.push(to)
    e.preventDefault()
  }

  return (
    <a
      href={url}
      onClick={onClick}
      target={newTab ? '_blank' : undefined}
      className={className}
    >
      {children}
    </a>
  )
}
