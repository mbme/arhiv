import { Store } from '@v/web-utils'
import { Gmail, IGmailMessage } from './gmail'

interface IState {
  messages: IGmailMessage[]
}

export class PoshtaStore extends Store<IState> {
  constructor(private _gmail: Gmail) {
    super({
      messages: [],
    })
  }

  async loadData() {
    const messages = await this._gmail.listMessages(undefined, 10).loadNextPage()

    this._setState({
      messages,
    })
  }
}
