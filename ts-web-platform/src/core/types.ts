import * as CSS from 'csstype'
import { theme } from './theme'

export type Tags = keyof HTMLElementTagNameMap

type CSSProps = CSS.Properties & CSS.PropertiesHyphen

type Spacing = keyof typeof theme.spacing
type Border = keyof typeof theme.border

interface ITransformerProps {
  margin?: CSS.Property.Margin<Spacing>
  marginTop?: CSS.Property.MarginTop<Spacing>
  marginRight?: CSS.Property.MarginRight<Spacing>
  marginBottom?: CSS.Property.MarginBottom<Spacing>
  marginLeft?: CSS.Property.MarginLeft<Spacing>

  m?: CSS.Property.Margin<Spacing>
  mx?: CSS.Property.MarginLeft<Spacing> | CSS.Property.MarginRight<Spacing>
  my?: CSS.Property.MarginTop<Spacing> | CSS.Property.MarginBottom<Spacing>
  mt?: CSS.Property.MarginTop<Spacing>
  mr?: CSS.Property.MarginRight<Spacing>
  mb?: CSS.Property.MarginBottom<Spacing>
  ml?: CSS.Property.MarginLeft<Spacing>

  padding?: CSS.Property.Padding<Spacing>
  paddingTop?: CSS.Property.PaddingTop<Spacing>
  paddingRight?: CSS.Property.PaddingRight<Spacing>
  paddingBottom?: CSS.Property.PaddingBottom<Spacing>
  paddingLeft?: CSS.Property.PaddingLeft<Spacing>

  p?: CSS.Property.Padding<Spacing>
  px?: CSS.Property.PaddingLeft<Spacing> | CSS.Property.PaddingRight<Spacing>
  py?: CSS.Property.PaddingTop<Spacing> | CSS.Property.PaddingBottom<Spacing>
  pt?: CSS.Property.PaddingTop<Spacing>
  pr?: CSS.Property.PaddingRight<Spacing>
  pb?: CSS.Property.PaddingBottom<Spacing>
  pl?: CSS.Property.PaddingLeft<Spacing>

  top?: CSS.Property.Top<Spacing> | 0
  left?: CSS.Property.Left<Spacing> | 0
  bottom?: CSS.Property.Bottom<Spacing> | 0
  right?: CSS.Property.Right<Spacing> | 0

  border?: CSS.Property.Border<Border>
  borderTop?: CSS.Property.BorderTop<Border>
  borderRight?: CSS.Property.BorderRight<Border>
  borderBottom?: CSS.Property.BorderBottom<Border>
  borderLeft?: CSS.Property.BorderLeft<Border>

  boxShadow?: keyof typeof theme.boxShadow | CSS.Property.BoxShadow

  width?: CSS.Property.Width<Spacing>
  height?: CSS.Property.Height<Spacing>

  fontSize?: CSS.Property.FontSize<keyof typeof theme.fontSize>
  zIndex?: keyof typeof theme.zIndex | CSS.Property.ZIndex

  bgColor?: CSS.Property.Color

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
  '&>*'?: CommonProps
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
