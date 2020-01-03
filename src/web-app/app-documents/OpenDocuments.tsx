import * as React from 'react'

interface IProps {
  ids: string[],
}

export function OpenDocuments({ ids }: IProps) {
  return (
    <div>
      {ids}
    </div>
  )
}
