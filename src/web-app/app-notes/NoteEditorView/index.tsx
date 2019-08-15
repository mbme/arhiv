import * as React from 'react'
import { Note as ArhivNote } from '~/arhiv'
import { Heading } from '~/web-platform'

import { NoteEditor } from './NoteEditor'
import { LockState } from '~/isodb/replica'

interface IProps {
  note: ArhivNote
}

interface IState {
  lock?: LockState
}

export class NoteEditorView extends React.PureComponent<IProps, IState> {
  state: IState = {
    lock: undefined,
  }

  private _unsubscribe?: () => void

  componentDidMount() {
    this._unsubscribe = this.props.note.$lock.subscribe((lock) => {
      if (lock.state === 'initial') {
        lock.acquire()
      }

      this.setState({ lock })
    })
  }

  componentWillUnmount() {
    const {
      lock,
    } = this.state

    if (this._unsubscribe) {
      this._unsubscribe()
    }

    if (lock && lock.state === 'pending') {
      lock.cancel()
    }

    if (lock && lock.state === 'acquired') {
      lock.release()
    }
  }

  render() {
    const {
      note,
    } = this.props

    const {
      lock,
    } = this.state

    if (!lock || lock.state !== 'acquired') {
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
