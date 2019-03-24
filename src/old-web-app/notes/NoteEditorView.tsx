import React, { PureComponent } from 'react'
import NotFoundView from '../chrome/NotFoundView'
import NoteEditor from './NoteEditor'
import {
  AppStore,
  inject,
  CoreTypes,
  IsodbReplica,
} from '../store'
import {
  ViewLayout,
} from '../parts'
import { ILocation } from '../../web-router'

interface IProps {
  id?: string
  note?: CoreTypes.INote
  push(route: ILocation): void
}

class NoteEditorView extends PureComponent<IProps> {
  closeEditor = () => {
    const {
      id,
      push,
    } = this.props

    push(
      id
        ? { path: '/note', params: { id } }
        : { path: '/notes', params: {} }
    )
  }

  onSave = () => {

  }

  render() {
    const {
      id,
      note,
    } = this.props

    if (id && !note) {
      return <NotFoundView />
    }

    return (
      <ViewLayout>
        <NoteEditor
          id={id}
          name={note ? note.name : ''}
          data={note ? note.data : ''}
          onSave={this.onSave}
          onCancel={this.closeEditor}
        />
      </ViewLayout>
    )
  }
}

const mapStoreToProps = (store: AppStore, props: Partial<IProps>, db: IsodbReplica) => ({
  note: props.id ? db.getRecord(props.id) : null,
  push: store.push,
})

export default inject(mapStoreToProps, NoteEditorView)
