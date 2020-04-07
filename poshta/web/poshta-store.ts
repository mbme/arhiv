import { Store } from '@v/web-utils'
import { GmailAPI, GmailMessage } from './gmail'

interface IState {
  messages: GmailMessage[]
  focusedIndex: number
  selected?: GmailMessage
}

export class PoshtaStore extends Store<IState> {
  constructor(private _gmail: GmailAPI) {
    super({
      messages: [],
      focusedIndex: -1,
      selected: undefined,
    })
  }

  async loadData() {
    const messages = await this._gmail.listMessages(undefined, 10).loadNextPage()

    this._setState({
      messages,
    })
  }

  focusNext() {
    let nextIndex = this.state.focusedIndex + 1
    if (nextIndex === this.state.messages.length) {
      nextIndex -= 1
    }

    this._setState({
      focusedIndex: nextIndex,
    })
  }

  focusPrev() {
    let prevIndex = this.state.focusedIndex - 1
    if (prevIndex < 0) {
      prevIndex = -1
    }

    this._setState({
      focusedIndex: prevIndex,
    })
  }

  selectFocused() {
    const {
      focusedIndex,
      messages,
    } = this.state

    this.select(messages[focusedIndex])
  }

  select(message?: GmailMessage) {
    this._setState({
      selected: message,
    })
  }
}
