import {
  Obj,
} from '~/utils'

export interface IStyleObject {
  [property: string]: IStyleObject | string | number | boolean | null | undefined
}

export type StyleRule = (props: Obj) => (IStyleObject | false | null | undefined)

export type Style = StyleRule | IStyleObject
