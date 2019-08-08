import * as React from 'react'
import { Note as ArhivNote } from '~/arhiv'
import { Heading } from '~/web-platform'

import { NoteEditor } from './NoteEditor'

interface IProps {
  note: ArhivNote
}

interface IState {
  hasLock: boolean
}

export class NoteEditorView extends React.PureComponent<IProps, IState> {
  state: IState = {
    hasLock: false,
  }

  async componentDidMount() {
    // FIXME cancel on unmount - remove from the queue
    await this.props.note.acquireLock()
    this.setState({ hasLock: true })
  }

  componentWillUnmount() {
    const {
      note,
    } = this.props

    if (note.lock) {
      note.lock.release()
    }
  }

  render() {
    const {
      note,
    } = this.props

    const {
      hasLock,
    } = this.state

    if (!hasLock) {
      return (
        <Heading>
          Note is in a read-only state, please wait
        </Heading>
      )
    }

    return (
      <NoteEditor note={note} />
    )
  }
}
