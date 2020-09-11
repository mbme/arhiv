import * as React from 'react'
import {
  ProgressLocker, StyleArg,
} from '@v/web-platform'
import { usePromise, RouterContext } from '@v/web-utils'
import { API } from './types'
import { Frame, ErrorBlock, NotFoundBlock, Action } from './parts'
import { Metadata } from './Metadata'
import { Note } from './Note'

const $container: StyleArg = {
  pt: 'medium',
}

interface IProps {
  id: string
}

export function Card({ id }: IProps) {
  const router = RouterContext.use()
  const [metadata, showMetadata] = React.useState(false)
  const [note, err] = usePromise(() => API.get_note(id), [id])

  if (err) {
    return (
      <ErrorBlock error={err} />
    )
  }

  if (note === undefined) {
    return (
      <ProgressLocker />
    )
  }

  if (note === null) {
    return (
      <NotFoundBlock>
        Can't find note with id "{id}"
      </NotFoundBlock>
    )
  }

  const actions = metadata ? (
    <Action
      type="action"
      onClick={() => showMetadata(false)}
    >
      Back
    </Action>
  ) : (
    <>
      <Action
        type="location"
        replace
        to={{ path: `/${id}/edit` }}
      >
        Edit Note
      </Action>

      <Action
        type="action"
        onClick={() => showMetadata(true)}
      >
        Show Metadata
      </Action>

      <Action
        type="action"
        onClick={() => router.goBack()}
      >
        Close
      </Action>
    </>
  )

  return (
    <Frame
      actions={actions}
      $style={$container}
    >
      {metadata ? (
        <Metadata document={note} />
      ) : (
        <Note name={note.data.name} data={note.data.data} />
      )}
    </Frame>
  )
}
