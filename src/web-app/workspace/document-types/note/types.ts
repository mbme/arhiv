import { Document } from '~/arhiv/replica'

interface IProps {
  name: string,
  data: string
}
export type DocumentNote = Document<IProps>
