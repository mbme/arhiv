import * as React from 'react'
import { CodeBlock } from '@v/web-platform'
import { usePromise } from '@v/web-utils'
import { API } from '../api'
import { FrameTitle } from '../parts'

export function StatusView() {
  const [status] = usePromise(() => API.get_status(), [])

  return (
    <>
      <FrameTitle>
        Status
      </FrameTitle>

      <CodeBlock>
        {status}
      </CodeBlock>
    </>
  )
}
