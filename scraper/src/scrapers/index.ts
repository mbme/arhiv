import { ArrayElement } from '../utils';
import { ExtractScraperGeneric } from './scraper';

import { scrapeFBPost } from './facebook/post';
import { scrapeFBPostList } from './facebook/post-list';
import { scrapeFBMobilePostList } from './facebook/post-list-mobile';
import { scrapeFBMobilePost } from './facebook/post-mobile';
import { scrapeBookFromYakaboo } from './yakaboo-book';
import { scrapeGameFromSteam } from './steam-game';
import { scrapeAnimeFromMyanimelist } from './myanimelist-anime';

export const SCRAPERS = [
  //
  scrapeFBPost,
  scrapeFBPostList,
  scrapeFBMobilePost,
  scrapeFBMobilePostList,
  scrapeBookFromYakaboo,
  scrapeGameFromSteam,
  scrapeAnimeFromMyanimelist,
] as const;

export type ScrapedData = ExtractScraperGeneric<ArrayElement<typeof SCRAPERS>>;
