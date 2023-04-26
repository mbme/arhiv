import { getEl, getTable, Obj, waitForFunction } from '../utils';
import { Scraper } from './scraper';

const LANGUAGE_TRANSLATIONS: Obj<string> = {
  'Українська': 'Ukrainian',
  'Англійська': 'English',
  'Російська': 'Russian',
};

export type YakabooBook = {
  typeName: 'YakabooBook';
  coverURL: string;
  title: string;
  authors: string;
  publicationDate: string;
  description: string;
  translators: string;
  publisher: string;
  pages: number;
  language: string;
};

export class YakabooBookScraper extends Scraper<'YakabooBook', YakabooBook> {
  // https://yakaboo.ua/ua/stories-of-your-life-and-others.html
  readonly pattern = new URLPattern({
    hostname: 'www.yakaboo.ua',
    pathname: '/ua/*',
  });

  readonly scrape = async (): Promise<YakabooBook> => {
    const coverURL = getEl<HTMLImageElement>('.gallery [aria-hidden=false] img', 'cover image').src;
    const title = getEl('.base-product__title h1', 'title')
      .innerText.substring('Книга '.length) // remove the prefix that Yakaboo adds to all titles
      .trim();

    getEl('.description__btn', 'expand description button').click();

    const description = getEl('.description__content', 'description').innerText;

    getEl('.main__chars button.ui-btn-nav', 'expand attributes button').click();
    await waitForFunction(
      () =>
        document
          .querySelector<HTMLElement>('.main__chars button.ui-btn-nav')
          ?.innerText.includes('Приховати') ?? false,
      'collapse attributes button'
    );

    const table = getTable(document, '.product-chars .chars .char', '\n');

    const authors = table['Автор'] || '';

    const language = LANGUAGE_TRANSLATIONS[table['Мова'] || ''];

    const publicationDate = table['Рік видання'] || '';
    const translators = table['Перекладач'] || '';
    const publisher = table['Видавництво'] || '';
    const pages = Number.parseInt(table['Кількість сторінок'] || '', 10);

    return {
      typeName: 'YakabooBook',
      coverURL,
      title,
      authors,
      publicationDate,
      description,
      translators,
      publisher,
      pages,
      language,
    };
  };
}
