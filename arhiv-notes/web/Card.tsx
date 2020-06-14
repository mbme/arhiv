import * as React from 'react'
import {
  Link,
  Row,
  ProgressLocker,
} from '@v/web-platform'
import { usePromise, RouterContext } from '@v/web-utils'
import { API } from './notes'
import { CloseIcon, Frame } from './parts'
import { Metadata } from './Metadata'
import { Note } from './Note'

interface IProps {
  id: string
}

export function Card({ id }: IProps) {
  const router = RouterContext.use()
  const [note] = usePromise(() => API.get_note(id), [id])

  if (!note) { // FIXME not found
    return (
      <ProgressLocker />
    )
  }

  const buttons = (
    <Row>
      <Link to={{ path: '/${id}/edit' }}>
        Edit
      </Link>

      <CloseIcon onClick={() => router.goBack() } />
    </Row>
  )

  const tabs = {
    [note.type]: () => <Note name={note.data.name} data={note.data.data} />,
    'metadata': () => <Metadata document={note} />,
  }

  return (
    <Frame
      tabs={tabs}
      buttons={buttons}
    />
  )
}
