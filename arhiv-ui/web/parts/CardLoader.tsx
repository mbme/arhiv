import * as React from 'react'
import {
  ProgressLocker,
} from '@v/web-platform'
import { Obj } from '@v/utils'
import { usePromise  } from '@v/web-utils'
import { API, IDocument } from '../api'
import { ErrorBlock, NotFoundBlock } from '../parts'

type Props<P extends Obj> = {
  id: string
  children(document: IDocument<string, P>): JSX.Element
} | {
  id: undefined
  createDocument(): Promise<IDocument<string, P>>
  children(document: IDocument<string, P>): JSX.Element
}

export function CardLoader<P>(props: Props<P>) {
  const [document, err] = usePromise(() => {
    if ('createDocument' in props) {
      return props.createDocument()
    }

    return API.get(props.id)
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
