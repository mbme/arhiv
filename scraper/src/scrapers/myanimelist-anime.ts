import { getEl, getTable } from '../utils';
import { Scraper } from './scraper';

export type MyAnimeListAnime = {
  typeName: 'MyAnimeListAnime';
  title: string;
  coverURL: string;
  releaseDate: string;
  creators: string;
  duration: string;
  description: string;
};

export class MyAnimeListAnimeScraper extends Scraper<'MyAnimeListAnime', MyAnimeListAnime> {
  // https://myanimelist.net/anime/30276/One_Punch_Man
  readonly pattern = new URLPattern({
    hostname: 'myanimelist.net',
    pathname: '/anime/*',
  });

  readonly scrape = (): MyAnimeListAnime => {
    const engTitle = getEl('.title-english')?.innerText;

    const title = engTitle || getEl('.title-name')?.innerText || '';

    const coverURL = getEl('.leftside img', 'cover image').dataset.src || '';

    const metadata = getTable(document, '.leftside .spaceit_pad');
    const releaseDate = metadata.Aired || '';
    const creators = metadata.Studios || '';
    const duration = metadata.Duration || '';

    const description = getEl('[itemprop=description]', 'description').innerText;

    const related = getTable(document, '.anime_detail_related_anime tr');
    if (related.Prequel) {
      throw new Error("Can't import an anime: it has a prequel. Start from the first season.");
    }

    return {
      typeName: 'MyAnimeListAnime',
      title,
      coverURL,
      releaseDate,
      creators,
      duration,
      description,
    };
  };
}
