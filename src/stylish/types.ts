export interface IStyleObject {
  [property: string]: IStyleObject | string | number | boolean | null | undefined
}

export type StyleTransformer = (src: IStyleObject) => IStyleObject

export interface IProps {
  [property: string]: any
}

export type StyleRuleResult = IStyleObject | false | null | undefined
export type StyleRule = (props: IProps) => StyleRuleResult

export type StylishDeclaration = IStyleObject | StyleRule
