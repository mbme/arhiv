import React, { PureComponent } from 'react';
import { classNames } from '../../utils';
import { Consumer, locationShape } from '../chrome/Router';
import './Link.css'

interface IProps {
  className?: string
  children: React.ReactNode
  clean?: boolean
}
export default class Link extends PureComponent<IProps, {}> {
  router = null;

  onClick = () => {
    this.router.push(this.props.to);
  };

  render() {
    const { className, children, clean } = this.props;

    return (
      <div
        className={classNames('Link', { 'is-clean': clean }, className)}
        role="link"
        tabIndex={0}
        onClick={this.onClick}
      >
        <Consumer>
          {(router) => {
            this.router = router;
          }}
        </Consumer>

        {children}
      </div>
    );
  }
}
