import * as React from 'react'
import {
  ProgressLocker,
} from '@v/web-platform'
import { usePromise  } from '@v/web-utils'
import { API, IDocument } from '../api'
import { ErrorBlock, NotFoundBlock } from '../parts'

type Props = {
  id: string
  children(document: IDocument): JSX.Element
} | {
  id: undefined
  createDocument(): Promise<IDocument>
  children(document: IDocument): JSX.Element
}

export function CardLoader(props: Props) {
  const [document, err] = usePromise(() => {
    if (props.id) {
      return API.get(props.id)
    }

    if ('createDocument' in props) {
      return props.createDocument()
    }

    throw new Error('id or createDocument prop must be provided')
  }, [])

  if (err) {
    return (
      <ErrorBlock error={err} />
    )
  }

  if (document === undefined) {
    return (
      <ProgressLocker />
    )
  }

  if (document === null) {
    return (
      <NotFoundBlock>
        {props.id ? (
          <>
            Can't find document with id "{props.id}"
          </>
        ) : (
          <>
            Failed to create document
          </>
        )}
      </NotFoundBlock>
    )
  }

  return props.children(document)
}
