import { createContext } from '~/web-utils'
import { ArhivReplica } from '~/arhiv/replica'

export const ArhivContext = createContext<ArhivReplica>()
