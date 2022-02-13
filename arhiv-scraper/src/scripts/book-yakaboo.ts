import { Context } from './context';
import { getTable, Obj } from './utils';

const LANGUAGE_TRANSLATIONS: Obj = {
  'Українська': 'Ukrainian',
  'Англійська': 'English',
  'Російська': 'Russian',
};

export async function extractBookFromYakaboo(url: string, context: Context): Promise<boolean> {
  // https://www.yakaboo.ua/ua/stories-of-your-life-and-others.html
  if (!url.includes('www.yakaboo.ua/ua/')) {
    return false;
  }

  const page = await context.newPage(url);

  const data: Obj = {};

  const cover_src = await page.$eval('#image', (node) => (node as HTMLImageElement).src);
  data.cover = await context.channel.createAttachment(cover_src);

  const title = await page.$eval(
    '#product-title h1',
    (node) => (node as HTMLHeadingElement).innerText
  );
  data.title = title.substring('Книга '.length); // remove the prefix that Yakaboo adds to all titles

  await page.$eval("a[href='#tab-description']", (node) => (node as HTMLAnchorElement).click());
  data.description = await page.$eval(
    '.description-shadow',
    (node) => (node as HTMLElement).innerText
  );

  await page.$eval("a[href='#tab-attributes']", (node) => (node as HTMLAnchorElement).click());

  const table = await getTable(page, '#product-attribute-specs-table tr', '\n');
  data.authors = table['Автор'];

  const language = LANGUAGE_TRANSLATIONS[table['Мова'] || ''];
  if (language) {
    data.language = language;
  }

  data.publication_date = table['Рік видання'];
  data.translators = table['Перекладач'];
  data.publisher = table['Видавництво'];
  data.pages = Number.parseInt(table['Кількість сторінок'] || '', 10);

  await context.channel.createDocument('book', data);

  return true;
}
