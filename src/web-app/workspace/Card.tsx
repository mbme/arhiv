import * as React from 'react'
import { useArhiv } from '~/arhiv/replica'
import {
  Box,
  ProgressLocker,
  useObservable,
} from '~/web-platform'
import { prettyPrintJSON } from '~/utils'
import {
  NotFound,
} from '../parts'

interface IProps {
  id: string
}

// Card
export function Card({ id }: IProps) {
  const arhiv = useArhiv()
  const [document, isReady] = useObservable(() => arhiv.documents.getDocument$(id), [id])

  if (!isReady) {
    return (
      <ProgressLocker />
    )
  }

  if (!document) {
    return NotFound
  }

  return (
    <Box
      height="100%"
      width="360px"
      overflow="auto"
      fontFamily="monospace"
      wordBreak="break-word"
      background="yellow"
    >
      {prettyPrintJSON(document.record)}
    </Box>
  )
}
