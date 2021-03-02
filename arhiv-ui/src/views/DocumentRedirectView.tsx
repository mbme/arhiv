import { Redirect } from '@v/web-utils'
import * as React from 'react'
import { CardLoader } from '../parts'

interface IProps {
  id: string
}

export function DocumentRedirectView({ id }: IProps) {
  return (
    <CardLoader id={id}>
      {document => (
        <Redirect
          to={{ path: `/catalog/${document.documentType}/${document.id}` }}
          replace
        />
      )}
    </CardLoader>
  )
}
