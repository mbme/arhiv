import React, { PureComponent, Fragment } from 'react';
import PropTypes from 'prop-types';
import { inject } from '../store';
import { ProgressLocker } from '../components';
import Redirect from '../parts/Redirect';

import AuthView from './AuthView';
import NotFoundView from './NotFoundView';
import ThemeView from './ThemeView';
import NotesView from '../notes/NotesView';
import NoteView from '../notes/NoteView';
import NoteEditorView from '../notes/NoteEditorView';
import { IRoute } from '../../web-router';

const Routes: { [key: string]: (params: any) => JSX.Element } = {
  '/': () => <Redirect to={{ name: 'notes' }} />,
  '/theme': () => <ThemeView />,
  '/notes': () => <NotesView />,
  '/note': ({ id }) => {
    if (!id) {
      return <NotFoundView />;
    }

    return <NoteView id={parseInt(id, 10)} />;
  },
  '/note-editor': ({ id }) => <NoteEditorView id={id ? parseInt(id, 10) : null} />,
};

interface IProps {
  route?: IRoute
  isAuthorized: boolean
  isLockerVisible: boolean
}

interface IState {
  view?: JSX.Element
}

class AppView extends PureComponent<IProps, IState> {
  static propTypes = {
    route: PropTypes.object,
    isAuthorized: PropTypes.bool,
    isLockerVisible: PropTypes.bool.isRequired,
  };

  state = {
    view: undefined,
  };

  static getDerivedStateFromProps({ route, isAuthorized }: IProps) {
    if (!isAuthorized) {
      return {
        view: <AuthView />,
      }
    }

    if (!route) {
      return {
        view: null,
      };
    }

    const getView = Routes[route.path];
    if (!getView) {
      return {
        view: <NotFoundView />,
      };
    }

    return {
      view: getView(route.params),
    };
  }


  render() {
    return (
      <Fragment>
        {this.state.view}
        {this.props.isLockerVisible && <ProgressLocker />}
      </Fragment>
    );
  }
}

const mapStoreToProps = (state, actions) => ({
  isLockerVisible: state.showLocker,
  isAuthorized: state.isAuthorized,
  route: state.route,
});

export default inject(mapStoreToProps, AppView);
