import { randomId } from '~/randomizer'

const ID_ALPHABET = '0123456789abcdefghijklmnopqrstuvwxyz'
const ID_LENGTH = 15

export const generateRecordId = () => randomId(ID_ALPHABET, ID_LENGTH)
