import { Browser } from 'puppeteer-core';
import type { Obj } from './utils';

const LANGUAGE_TRANSLATIONS: Obj = {
  'Українська': 'Ukrainian',
  'Англійська': 'English',
  'Російська': 'Russian',
};

export async function extractBookFromYakaboo(url: string, browser: Browser): Promise<Obj | undefined> {
  if (!url.includes('www.yakaboo.ua/ua/')) {
    return undefined;
  }

  const page = await browser.newPage();
  await page.goto(url);

  const data: Obj = {
    title: await page.$eval('#product-title h1', node => (node as HTMLHeadingElement).innerText),
    cover_src: await page.$eval('#image', node => (node as HTMLImageElement).src),
  };

  await page.$eval("a[href='#tab-description']", node => (node as HTMLAnchorElement).click());
  data['description'] = await page.$eval('.description-shadow', node => (node as HTMLElement).innerText);

  await page.$eval("a[href='#tab-attributes']", node => (node as HTMLAnchorElement).click());

  for (const row of await page.$$('#product-attribute-specs-table tr')) {
    const [prop, value] = await row.$$eval('td', nodes => nodes.map(node => (node as HTMLTableRowElement).innerText));

    switch (prop) {
      case 'Автор': {
        data['authors'] = value;
        break;
      }

      case 'Мова': {
        const language = LANGUAGE_TRANSLATIONS[value];
        if (language) {
          data['language'] = language;
        }
        break;
      }

      case 'Рік видання': {
        data['publication_date'] = value;
        break;
      }

      case 'Перекладач': {
        data['translators'] = value;
        break;
      }

      case 'Видавництво': {
        data['publisher'] = value;
        break;
      }

      case 'Кількість сторінок': {
        data['pages'] = Number.parseInt(value, 10);
        break;
      }

      case 'ISBN': {
        data['ISBN'] = value;
        break;
      }

      default: break;
    }
  }

  return data;
}
