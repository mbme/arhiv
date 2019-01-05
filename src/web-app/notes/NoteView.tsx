import React, { PureComponent, Fragment } from 'react';
import {
  inject,
  IStoreState,
  ActionsType,
  IsodbReplica,
  Note as NoteType,
} from '../store'
import { Icon } from '../components';
import { Link, Toolbar } from '../parts';
import NotFoundView from '../chrome/NotFoundView';
import DeleteNoteButton from './DeleteNoteButton';
import Note from './Note';

interface IProps {
  note?: NoteType
}

class NoteView extends PureComponent<IProps> {
  render() {
    const { note } = this.props;

    if (!note) return <NotFoundView />;

    const deleteBtn = (
      <DeleteNoteButton key="delete" id={note._id} />
    );

    const editBtn = (
      <Link to={{ path: 'note-editor', params: { id: note._id } }} clean>
        <Icon type="edit-2" />
      </Link>
    );

    return (
      <Fragment>
        <Toolbar left={deleteBtn} right={editBtn} />
        <Note name={note.name} data={note.data} />
      </Fragment>
    );
  }
}

const mapStoreToProps = (state: IStoreState, actions: ActionsType, db: IsodbReplica) => ({
  notes: db.getRecords(),
})

export default inject(mapStoreToProps, NoteView)
