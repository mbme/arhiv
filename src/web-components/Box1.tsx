import * as React from 'react'
import { Obj } from '~/utils';
import theme from './theme';

const getSpacing = (val: any) => (theme.spacing as any)[val] || val

const Rules = {
  m: (props: Obj) => ({
    margin: getSpacing(props.m),
  }),
  mx: (props: Obj) => ({
    marginLeft: getSpacing(props.mx),
    marginRight: getSpacing(props.mx),
  }),
  my: (props: Obj) => ({
    marginTop: getSpacing(props.my),
    marginBottom: getSpacing(props.my),
  }),
  mt: (props: Obj) => ({
    marginTop: getSpacing(props.mt),
  }),
  mr: (props: Obj) => ({
    marginRight: getSpacing(props.mr),
  }),
  mb: (props: Obj) => ({
    marginBottom: getSpacing(props.mb),
  }),
  ml: (props: Obj) => ({
    marginLeft: getSpacing(props.ml),
  }),

  p: (props: Obj) => ({
    padding: getSpacing(props.p),
  }),
  px: (props: Obj) => ({
    paddingLeft: getSpacing(props.px),
    paddingRight: getSpacing(props.px),
  }),
  py: (props: Obj) => ({
    paddingTop: getSpacing(props.py),
    paddingBottom: getSpacing(props.py),
  }),
  pt: (props: Obj) => ({
    paddingTop: getSpacing(props.pt),
  }),
  pr: (props: Obj) => ({
    paddingRight: getSpacing(props.pr),
  }),
  pb: (props: Obj) => ({
    paddingBottom: getSpacing(props.pb),
  }),
  pl: (props: Obj) => ({
    paddingLeft: getSpacing(props.pl),
  }),

  top: (props: Obj) => ({
    top: getSpacing(props.top),
  }),
  left: (props: Obj) => ({
    left: getSpacing(props.left),
  }),
  bottom: (props: Obj) => ({
    bottom: getSpacing(props.bottom),
  }),
  right: (props: Obj) => ({
    right: getSpacing(props.right),
  }),

  fontSize: (props: Obj) => ({
    fontSize: (theme.fontSize as any)[props.fontSize] || props.fontSize,
  }),
  fontFamily: (props: Obj) => ({
    fontFamily: (theme.fontFamily as any)[props.fontFamily] || props.fontFamily,
  }),
  zIndex: (props: Obj) => ({
    zIndex: (theme.zIndex as any)[props.zIndex],
  }),
}

export class Box extends React.PureComponent<Obj> {
  render() {
    const {
      as = 'div',
      className,
      children,

      ...props
    } = this.props

    return React.createElement(as, { className: classes(getStyles(props), className) }, children)
  }
}
