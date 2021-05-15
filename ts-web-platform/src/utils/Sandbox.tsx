import * as React from 'react'
import { noop } from '@v/utils'
import { HotkeysResolverContext } from './hotkeys'

interface IProps {
  content: string
  loadRemoteContent?: boolean
}

export function Sandbox({ content, loadRemoteContent }: IProps) {
  const iframeRef = React.useRef<HTMLIFrameElement>(null)
  const [height, setHeight] = React.useState(600)
  const hotkeysResolver = HotkeysResolverContext.use()

  React.useEffect(() => {
    if (!iframeRef.current) {
      return noop
    }

    const document = iframeRef.current.contentDocument!

    return hotkeysResolver.registerDocument(document)
  }, [iframeRef.current])

  React.useEffect(() => {
    const document = iframeRef.current!.contentDocument!

    if (!loadRemoteContent) {
      const el = document.createElement('meta')
      el.setAttribute('http-equiv', 'Content-Security-Policy')
      el.setAttribute('content', "default-src 'self'; style-src 'unsafe-inline'")

      document.head.appendChild(el)
    }

    document.body.innerHTML = content
    setHeight(document.body.scrollHeight)

    const resizeObserver = new ResizeObserver(() => {
      setHeight(document.body.scrollHeight)
    })

    resizeObserver.observe(document.body)

    return () => {
      resizeObserver.unobserve(document.body)
    }
  }, [])

  return (
    <iframe
      ref={iframeRef}
      sandbox=""
      width="100%"
      height={height}
      scrolling="no"
      frameBorder="0"
    />
  )
}
