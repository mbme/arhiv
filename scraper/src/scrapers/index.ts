import { ArrayElement } from '../utils';
import { ExtractScraperGeneric } from './scraper';

import { scrapeFBPost } from './facebook/post';
import { scrapeFBPostList } from './facebook/post-list';
import { scrapeFBMobilePostList } from './facebook/post-list-mobile';
import { scrapeFBMobilePost } from './facebook/post-mobile';
import { scrapeBookFromYakaboo } from './yakaboo-book';

export const SCRAPERS = [
  //
  scrapeFBPost,
  scrapeFBPostList,
  scrapeFBMobilePost,
  scrapeFBMobilePostList,
  scrapeBookFromYakaboo,
] as const;

export type ScrapedData = ExtractScraperGeneric<ArrayElement<typeof SCRAPERS>>;
