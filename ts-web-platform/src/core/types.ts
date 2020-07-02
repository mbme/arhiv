import * as CSS from 'csstype'
import { theme } from './theme'

export type Tags = keyof HTMLElementTagNameMap

type CSSProps = CSS.Properties & CSS.PropertiesHyphen

type Spacing = keyof typeof theme.spacing
type Border = keyof typeof theme.border
type Color = keyof typeof theme.color | CSS.Color

interface ITransformerProps {
  margin?: CSS.MarginProperty<Spacing>
  marginTop?: CSS.MarginTopProperty<Spacing>
  marginRight?: CSS.MarginRightProperty<Spacing>
  marginBottom?: CSS.MarginBottomProperty<Spacing>
  marginLeft?: CSS.MarginLeftProperty<Spacing>

  m?: CSS.MarginProperty<Spacing>
  mx?: CSS.MarginLeftProperty<Spacing> | CSS.MarginRightProperty<Spacing>
  my?: CSS.MarginTopProperty<Spacing> | CSS.MarginBottomProperty<Spacing>
  mt?: CSS.MarginTopProperty<Spacing>
  mr?: CSS.MarginRightProperty<Spacing>
  mb?: CSS.MarginBottomProperty<Spacing>
  ml?: CSS.MarginLeftProperty<Spacing>

  padding?: CSS.PaddingProperty<Spacing>
  paddingTop?: CSS.PaddingTopProperty<Spacing>
  paddingRight?: CSS.PaddingRightProperty<Spacing>
  paddingBottom?: CSS.PaddingBottomProperty<Spacing>
  paddingLeft?: CSS.PaddingLeftProperty<Spacing>

  p?: CSS.PaddingProperty<Spacing>
  px?: CSS.PaddingLeftProperty<Spacing> | CSS.PaddingRightProperty<Spacing>
  py?: CSS.PaddingTopProperty<Spacing> | CSS.PaddingBottomProperty<Spacing>
  pt?: CSS.PaddingTopProperty<Spacing>
  pr?: CSS.PaddingRightProperty<Spacing>
  pb?: CSS.PaddingBottomProperty<Spacing>
  pl?: CSS.PaddingLeftProperty<Spacing>

  top?: CSS.TopProperty<Spacing> | 0
  left?: CSS.LeftProperty<Spacing> | 0
  bottom?: CSS.BottomProperty<Spacing> | 0
  right?: CSS.RightProperty<Spacing> | 0

  border?: CSS.BorderProperty<Border>
  borderTop?: CSS.BorderTopProperty<Border>
  borderRight?: CSS.BorderRightProperty<Border>
  borderBottom?: CSS.BorderBottomProperty<Border>
  borderLeft?: CSS.BorderLeftProperty<Border>

  boxShadow?: keyof typeof theme.boxShadow | CSS.BoxShadowProperty

  width?: CSS.WidthProperty<Spacing>
  height?: CSS.HeightProperty<Spacing>

  fontSize?: CSS.FontSizeProperty<keyof typeof theme.fontSize>
  fontFamily?: keyof typeof theme.fontFamily | CSS.FontFamilyProperty
  zIndex?: keyof typeof theme.zIndex | CSS.ZIndexProperty

  color?: Color
  bgColor?: Color
  backgroundColor?: Color

  relative?: boolean
  absolute?: boolean
  bold?: boolean
  uppercase?: boolean
  hidden?: boolean
}

type CommonProps = Omit<CSSProps, keyof ITransformerProps> & ITransformerProps

interface IStylishProps {
  '&:hover'?: CommonProps
  '&:focus'?: CommonProps
  '&:before'?: CommonProps
  '&:first-child'?: CommonProps
}

type CommonAndCustomProps = CommonProps & IStylishProps

interface IMediaQueryTransformerProps {
  fromSm?: CommonAndCustomProps
  fromMd?: CommonAndCustomProps
  fromLg?: CommonAndCustomProps
}

export type StyleProps = CommonAndCustomProps & IMediaQueryTransformerProps

export type StyleArg = StyleProps | undefined | null | false | ''

export interface IKeyframeProps {
  from?: CommonProps
  to?: CommonProps

  [percent: string]: CommonProps | undefined
}
