import {
  IGmailMessage,
  IGmailMessagePayload,
} from './types'
import { splitMessageParts, decodeData } from './utils'

export class GmailMessage {
  private readonly _textPart?: IGmailMessagePayload
  private readonly _htmlPart?: IGmailMessagePayload
  readonly attachments: IGmailMessagePayload[] = []

  constructor(public readonly data: IGmailMessage) {
    const {
      textPart,
      htmlPart,
      attachments,
    } = splitMessageParts(data.payload)

    this._textPart = textPart
    this._htmlPart = htmlPart
    this.attachments = attachments
  }

  getHeader(name: string): string[] {
    return this.data.payload.headers
      .filter(item => item.name === name)
      .map(item => item.value)
  }

  getSingleHeader(name: string): string {
    const values = this.getHeader(name)

    if (values.length !== 1) {
      throw new Error(`Found ${values.length} headers "${name}", expected 1`)
    }

    return values[0]
  }

  get date(): Date {
    return new Date(this.getSingleHeader('Date'))
  }

  get subject(): string {
    return this.getSingleHeader('Subject')
  }

  get from(): string {
    return this.getSingleHeader('From')
  }

  get to(): string {
    return this.getSingleHeader('To')
  }
  get id(): string {
    return this.data.id
  }

  get labels(): string[] {
    return this.data.labelIds
  }

  get snippet(): string {
    return this.data.snippet
  }

  hasHTMLBody(): boolean {
    return !!this._htmlPart
  }

  hasTextBody(): boolean {
    return !!this._textPart
  }

  getHTMLBody(): string {
    if (!this._htmlPart) {
      throw new Error('Message has no html part')
    }

    return decodeData(this._htmlPart.body.data)
  }

  getTextBody(): string {
    if (!this._textPart) {
      throw new Error('Message has no text part')
    }

    return decodeData(this._textPart.body.data)
  }
}
