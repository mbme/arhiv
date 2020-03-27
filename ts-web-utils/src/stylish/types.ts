export interface IStyleObject {
  [property: string]: IStyleObject | string | number | boolean | null | undefined
}

export type StyleTransformer = (src: IStyleObject) => IStyleObject
