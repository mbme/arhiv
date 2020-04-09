import { base64url2base64 } from '@v/utils'
import {
  IGmailMessagePayload,
} from './types'


export function decodeData(data: string): string {
  return Buffer.from(base64url2base64(data), 'base64').toString('utf8')
}

function isTextPart(part: IGmailMessagePayload): boolean {
  return part.mimeType === 'text/plain'
}

function isHTMLPart(part: IGmailMessagePayload): boolean {
  return part.mimeType === 'text/html'
}

function isAlternativePart(part: IGmailMessagePayload): boolean {
  return part.mimeType === 'multipart/alternative'
}

interface ISplitResult {
  textPart?: IGmailMessagePayload
  htmlPart?: IGmailMessagePayload
  attachments: IGmailMessagePayload[]
}

function splitAlternativePart(alternativePart: IGmailMessagePayload): ISplitResult {
  if (!alternativePart.parts?.length)  {
    throw new Error('multipart/alternative has no parts')
  }

  let textPart: IGmailMessagePayload | undefined
  let htmlPart: IGmailMessagePayload | undefined

  for (const part of alternativePart.parts) {
    if (isTextPart(part)) {
      if (textPart) {
        throw new Error('multipart/alternative has duplicate text/plain part')
      }

      textPart = part

      continue
    }

    if (isHTMLPart(part)) {
      if (htmlPart) {
        throw new Error('multipart/alternative has duplicate text/html part')
      }

      htmlPart = part

      continue
    }

    throw new Error(`multipart/alternative has unexpected part ${part.mimeType}`)
  }

  return {
    textPart,
    htmlPart,
    attachments: [],
  }
}

function splitMixedPart(mixedPart: IGmailMessagePayload): ISplitResult {
  if (!mixedPart.parts?.length)  {
    throw new Error('multipart/mixed has no parts')
  }

  let textPart: IGmailMessagePayload | undefined
  let htmlPart: IGmailMessagePayload | undefined

  const firstPart = mixedPart.parts[0]
  if (isAlternativePart(firstPart)) {
    const result = splitAlternativePart(firstPart)
    textPart = result.textPart
    htmlPart = result.htmlPart
  }

  if (isTextPart(firstPart)) {
    textPart = firstPart
  }

  if (isHTMLPart(firstPart)) {
    htmlPart = firstPart
  }

  return {
    textPart,
    htmlPart,
    attachments: mixedPart.parts.slice(1),
  }
}

export function splitMessageParts(payload: IGmailMessagePayload): ISplitResult {
  if (isTextPart(payload)) {
    return {
      textPart: payload,
      htmlPart: undefined,
      attachments: [],
    }
  }

  if (isHTMLPart(payload)) {
    return {
      textPart: undefined,
      htmlPart: payload,
      attachments: [],
    }
  }

  if (isAlternativePart(payload)) {
    return splitAlternativePart(payload)
  }

  if (payload.mimeType === 'multipart/mixed') {
    return splitMixedPart(payload)
  }

  throw new Error(`message payload has unexpected mime type ${payload.mimeType}`)
}
