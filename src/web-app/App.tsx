import React, { PureComponent, Fragment } from 'react'
import { IRoute } from '../web-router'
import { inject, StateType } from './store'
import { ProgressLocker } from './components'
import Redirect from './parts/Redirect'

import AuthView from './chrome/AuthView'
import NotFoundView from './chrome/NotFoundView'
import ThemeView from './chrome/ThemeView'
import NotesView from './notes/NotesView'
import NoteView from './notes/NoteView'
import NoteEditorView from './notes/NoteEditorView'

const Routes: { [key: string]: (params: { [key: string]: string }) => JSX.Element } = {
  '/': () => <Redirect to={{ path: '/notes' }} />,
  '/theme': () => <ThemeView />,
  '/notes': () => <NotesView />,
  '/note': ({ id }) => {
    if (!id) {
      return <NotFoundView />
    }

    return <NoteView id={parseInt(id, 10)} />
  },
  '/note-editor': ({ id }) => <NoteEditorView id={id ? parseInt(id, 10) : undefined} />,
}

interface IProps {
  route?: IRoute
  isAuthorized: boolean
  isLockerVisible: boolean
}

interface IState {
  view?: JSX.Element
}

class App extends PureComponent<IProps, IState> {
  static getDerivedStateFromProps({ route, isAuthorized }: IProps) {
    if (!isAuthorized) {
      return {
        view: <AuthView />,
      }
    }

    if (!route) {
      return {
        view: undefined,
      }
    }

    const getView = Routes[route.path]
    if (!getView) {
      return {
        view: <NotFoundView />,
      }
    }

    return {
      view: getView(route.params),
    }
  }

  state = {
    view: undefined,
  }

  render() {
    return (
      <Fragment>
        {this.state.view}
        {this.props.isLockerVisible && <ProgressLocker />}
      </Fragment>
    )
  }
}

const mapStoreToProps = (state: StateType) => ({
  isLockerVisible: state.isLockerVisible,
  isAuthorized: state.isAuthorized,
  route: state.route,
})

export default inject(mapStoreToProps, App)
