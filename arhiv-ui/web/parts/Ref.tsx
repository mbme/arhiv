import * as React from 'react'
import { Link } from '@v/web-platform'

interface IProps {
  id: string
}

export function Ref({ id }: IProps) {
  return (
    <Link to={`/documents/${id}`}>
      {id}
    </Link>
  )
}
