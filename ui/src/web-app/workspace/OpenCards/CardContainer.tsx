import * as React from 'react'
import { Document } from '~/arhiv/replica'
import { noop } from '~/utils'
import { Box, ProgressLocker } from '~/web-platform'
import { getModule } from '../modules'

interface IProps {
  document: Document
  focused: boolean
}

export function CardContainer({ document, focused }: IProps) {
  const ref = React.useRef<HTMLDivElement | undefined>(undefined)

  React.useEffect(() => {
    if (!focused) {
      return noop
    }

    // we use timeout here cause cards render images
    // which change scroll height and results into card not being visible
    // FIXME use context ImageMonitorProvider and make Images notify it when they're loaded
    const timeoutId = setTimeout(() => {
      ref.current?.scrollIntoView(true)
    }, 100)

    return () => clearTimeout(timeoutId)
  }, [focused])

  if (!document) {
    return (
      <ProgressLocker />
    )
  }

  return (
    <Box
      flex="0 0 auto"
      mb="large"
      width="35rem"
      maxWidth="100%"
      innerRef={ref}
    >
      {getModule(document.type).renderCard(document)}
    </Box>
  )
}
