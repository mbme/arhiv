import { randomId } from '~/randomizer'

const ID_ALPHABET = '0123456789abcdefghijklmnopqrstuvwxyz'
const ID_LENGTH = 15

export const generateRandomId = () => randomId(ID_ALPHABET, ID_LENGTH)
