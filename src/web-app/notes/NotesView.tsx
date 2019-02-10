import React, { PureComponent } from 'react'
import { formatTs } from '../../utils/date'
import {
  inject,
  IsodbReplica,
  CoreTypes,
  AppStore,
} from '../store'
import {
  Button,
  Filter,
} from '../components'
import {
  Toolbar,
  Link,
  ViewLayout,
} from '../parts'
import './NotesView.css'

interface IProps {
  filter: string
  replaceParam(param: string, value: string): void
  notes: CoreTypes.INote[]
}

class NotesView extends PureComponent<IProps> {
  render() {
    const {
      filter,
      replaceParam,
      notes,
    } = this.props

    const items = notes.map(note => (
      <Link key={note._id} clean to={{ path: '/note', params: { id: note._id } }} className="Notes-link">
        <small className="Notes-ts">
          {formatTs(note.updatedTs)}
        </small>
        {note.name}
      </Link>
    ))

    const left = (
      <Filter
        placeholder="Filter notes"
        filter={filter}
        onChange={newFilter => replaceParam('filter', newFilter)}
      />
    )

    const addBtn = (
      <Link to={{ path: '/note-editor' }}>
        <Button primary>Add</Button>
      </Link>
    )

    return (
      <ViewLayout>
        <Toolbar left={left} right={addBtn} />

        <small className="Notes-counter">
          {items.length} items
        </small>

        {items}
      </ViewLayout>
    )
  }
}

const mapStoreToProps = (store: AppStore, _props: Partial<IProps>, db: IsodbReplica) => ({
  filter: store.state.route!.params.filter || '',
  replaceParam: store.replaceParam,
  notes: db.getRecords(),
})

export default inject(mapStoreToProps, NotesView)
