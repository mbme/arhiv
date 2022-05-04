import * as readline from 'readline';
import { once } from 'node:events';

type ScraperAction =
  | {
      type: 'CreateAttachment';
      url: string;
    }
  | {
      type: 'CreateDocument';
      document_type: string;
      subtype: string;
      data: Record<string, unknown>;
    };

export class ActionChannel {
  private _rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  private async runAction(action: ScraperAction): Promise<string> {
    console.log(JSON.stringify(action));

    const [value] = (await once(this._rl, 'line')) as string[];

    if (value === 'error') {
      throw new Error(`scraper action ${action.type} failed`);
    }

    return value;
  }

  async createAttachment(url: string) {
    if (!url) {
      return '';
    }

    return this.runAction({
      type: 'CreateAttachment',
      url,
    });
  }

  createDocument(document_type: string, subtype: string, data: Record<string, unknown>) {
    return this.runAction({
      type: 'CreateDocument',
      document_type,
      subtype,
      data,
    });
  }

  close() {
    this._rl.close();
  }
}
