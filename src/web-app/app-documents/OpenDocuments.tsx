import * as React from 'react'
import { OpenDocument } from './OpenDocument'

interface IProps {
  ids: string[],
}

export function OpenDocuments({ ids }: IProps) {
  return (
    <>
      {ids.map(id => (<OpenDocument key={id} id={id} />))}
    </>
  )
}
