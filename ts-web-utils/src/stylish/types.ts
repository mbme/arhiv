export interface IStyleObject {
  [property: string]: IStyleObject | string | number | boolean | null | undefined
}

export type StyleArg = IStyleObject | undefined | null | false | ''

export type StyleTransformer = (src: IStyleObject) => IStyleObject
