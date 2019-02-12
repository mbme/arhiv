import React, { PureComponent } from 'react'
import {
  inject,
  AppStore,
  IsodbReplica,
  CoreTypes,
} from '../store'
import { Icon } from '../components'
import {
  Link,
  Toolbar,
  ViewLayout,
} from '../parts'
import NotFoundView from '../chrome/NotFoundView'
import DeleteNoteButton from './DeleteNoteButton'
import Note from './Note'

interface IProps {
  id: string
  note?: CoreTypes.INote
}

class NoteView extends PureComponent<IProps> {
  render() {
    const { note } = this.props

    if (!note) return <NotFoundView />

    const deleteBtn = (
      <DeleteNoteButton key="delete" id={note._id} />
    )

    const editBtn = (
      <Link to={{ path: '/note-editor', params: { id: note._id } }} clean>
        <Icon type="edit-2" />
      </Link>
    )

    return (
      <ViewLayout>
        <Toolbar left={deleteBtn} right={editBtn} />

        <Note name={note.name} data={note.data} />
      </ViewLayout>
    )
  }
}

const mapStoreToProps = (_store: AppStore, props: Partial<IProps>, db: IsodbReplica) => ({
  note: db.getRecord(props.id!),
})

export default inject(mapStoreToProps, NoteView)
