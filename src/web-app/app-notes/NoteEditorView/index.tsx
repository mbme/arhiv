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

  private _unsubscribe?: () => void

  componentDidMount() {
    this._unsubscribe = this.props.note.$lock().subscribe((hasLock) => {
      this.setState({ hasLock })
    })
  }

  componentWillUnmount() {
    if (this._unsubscribe) {
      this._unsubscribe()
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
