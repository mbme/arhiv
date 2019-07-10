import { randomId } from '~/randomizer'
import { IChangeset } from './types'

const ID_ALPHABET = '0123456789abcdefghijklmnopqrstuvwxyz'
const ID_LENGTH = 15

export const generateRandomId = () => randomId(ID_ALPHABET, ID_LENGTH)

export const isEmptyChangeset = (changeset: IChangeset) => !changeset.records.length && !changeset.attachments.length
