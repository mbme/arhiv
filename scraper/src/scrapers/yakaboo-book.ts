import { getEl, getPathSegments, getTable, Obj } from '../utils';
import { Scraper } from './scraper';

const LANGUAGE_TRANSLATIONS: Obj<string> = {
  'Українська': 'Ukrainian',
  'Англійська': 'English',
  'Російська': 'Russian',
};

export type YakabooBook = {
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

// https://www.yakaboo.ua/ua/stories-of-your-life-and-others.html
export const scrapeBookFromYakaboo: Scraper<YakabooBook> = (locationURL) => {
  if (locationURL.hostname !== 'www.yakaboo.ua' || getPathSegments(locationURL)[0] !== 'ua') {
    return undefined;
  }

  const coverURL = getEl<HTMLImageElement>('#image', 'cover image').src;
  const title = getEl('#product-title h1', 'title').innerText.substring('Книга '.length); // remove the prefix that Yakaboo adds to all titles

  getEl("a[href='#tab-description']", 'description tab').click();

  const description = getEl('.description-shadow', 'description').innerText;

  getEl("a[href='#tab-attributes']", 'attributes tab').click();

  const table = getTable(document, '#product-attribute-specs-table tr', '\n');
  const authors = table['Автор'] || '';

  const language = LANGUAGE_TRANSLATIONS[table['Мова'] || ''];

  const publicationDate = table['Рік видання'] || '';
  const translators = table['Перекладач'] || '';
  const publisher = table['Видавництво'] || '';
  const pages = Number.parseInt(table['Кількість сторінок'] || '', 10);

  return {
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
