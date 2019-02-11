import React, { PureComponent, Fragment } from 'react'
import {
  inject,
  AppStore,
} from '../store'
import { Icon, ConfirmationDialog } from '../components'

interface IProps {
  id: string
  pushTo(path: string): void
  deleteRecord(id: string): void
}

interface IState {
  showConfirmation: boolean
}

class DeleteNoteButton extends PureComponent<IProps, IState> {
  state = {
    showConfirmation: false,
  }

  showConfirmationDialog = () => this.setState({ showConfirmation: true })
  hideConfirmationDialog = () => this.setState({ showConfirmation: false })

  deleteNote = () => {
    const {
      id,
      deleteRecord,
      pushTo,
    } = this.props

    deleteRecord(id)
    pushTo('/notes')
  }

  render() {
    return (
      <Fragment>
        <Icon
          title="Delete note"
          type="trash-2"
          onClick={this.showConfirmationDialog}
        />

        {this.state.showConfirmation && (
          <ConfirmationDialog
            confirmation="Delete"
            onConfirmed={this.deleteNote}
            onCancel={this.hideConfirmationDialog}
          >
            Are you sure you want to <b>delete this note?</b>
          </ConfirmationDialog>
        )
        }
      </Fragment>
    )
  }
}

const mapStoreToProps = (store: AppStore) => ({
  pushTo: store.pushTo,
  deleteRecord: store.deleteRecord,
})

export default inject(mapStoreToProps, DeleteNoteButton)
