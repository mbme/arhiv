import { randomId } from '../randomizer'
import { IAttachment, IChangedAttachment } from './types'

const ID_ALPHABET = '0123456789abcdefghijklmnopqrstuvwxyz'
const ID_LENGTH = 15

export const getRandomId = () => randomId(ID_ALPHABET, ID_LENGTH)

export const isAttachment = (item: any): item is (IAttachment | IChangedAttachment) => item._attachment || false
export const isDeleted = (item: any): boolean => item._deleted || false
