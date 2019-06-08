import * as React from 'react'
import {
  style,
  classes,
  types,
} from 'typestyle'
import theme from './theme'

type SpacingType = keyof typeof theme.spacing

interface IStyle extends Pick<types.CSSProperties, 'position'> {
  m?: SpacingType
  mx?: SpacingType
  my?: SpacingType
  mt?: SpacingType
  mr?: SpacingType
  mb?: SpacingType
  ml?: SpacingType

  p?: SpacingType
  px?: SpacingType
  py?: SpacingType
  pt?: SpacingType
  pr?: SpacingType
  pb?: SpacingType
  pl?: SpacingType

  top?: SpacingType
  left?: SpacingType
  bottom?: SpacingType
  right?: SpacingType

  color?: keyof typeof theme.color
  bg?: keyof typeof theme.color

  fontSize?: keyof typeof theme.fontSize
  fontFamily?: keyof typeof theme.fontFamily

  zIndex?: keyof typeof theme.zIndex
}

const getStyles = (props: IStyle) => style(
  props.m && {
    margin: theme.spacing[props.m],
  },
  props.mx && {
    marginLeft: theme.spacing[props.mx],
    marginRight: theme.spacing[props.mx],
  },
  props.my && {
    marginTop: theme.spacing[props.my],
    marginBottom: theme.spacing[props.my],
  },
  props.mt && {
    marginTop: theme.spacing[props.mt],
  },
  props.mr && {
    marginRight: theme.spacing[props.mr],
  },
  props.mb && {
    marginBottom: theme.spacing[props.mb],
  },
  props.ml && {
    marginLeft: theme.spacing[props.ml],
  },

  props.p && {
    padding: theme.spacing[props.p],
  },
  props.px && {
    paddingLeft: theme.spacing[props.px],
    paddingRight: theme.spacing[props.px],
  },
  props.py && {
    paddingTop: theme.spacing[props.py],
    paddingBottom: theme.spacing[props.py],
  },
  props.pt && {
    paddingTop: theme.spacing[props.pt],
  },
  props.pr && {
    paddingRight: theme.spacing[props.pr],
  },
  props.pb && {
    paddingBottom: theme.spacing[props.pb],
  },
  props.pl && {
    paddingLeft: theme.spacing[props.pl],
  },

  props.top && {
    top: theme.spacing[props.top],
  },
  props.left && {
    left: theme.spacing[props.left],
  },
  props.bottom && {
    bottom: theme.spacing[props.bottom],
  },
  props.right && {
    right: theme.spacing[props.right],
  },

  props.color && {
    color: theme.color[props.color],
  },
  props.bg && {
    background: theme.color[props.bg],
  },

  props.fontSize && {
    fontSize: theme.fontSize[props.fontSize],
  },
  props.fontFamily && {
    fontFamily: theme.fontFamily[props.fontFamily],
  },

  props.zIndex && {
    zIndex: theme.zIndex[props.zIndex],
  },

  props.position && {
    position: props.position,
  },
)

interface IProps extends IStyle {
  as?: string,
  className?: string,
  children?: React.ReactNode
}

export class Box extends React.PureComponent<IProps> {
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
