import * as React from 'react'
import { CodeBlock, usePromise } from '@v/web-platform'
import { API } from '@v/arhiv-api'
import { FrameTitle, useActions } from '../parts'

export function StatusView() {
  const [lastSyncTime, setLastSyncTime] = React.useState(0)
  const [status] = usePromise(() => API.get_status(), [lastSyncTime])

  useActions(() => [
    {
      onClick: () => {
        API.sync().finally(() => setLastSyncTime(Date.now()))
      },
      children: 'Sync',
    },
  ])

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
