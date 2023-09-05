import {
  getAll,
  getEl,
  getListStr,
  getListValues,
  scrollToBottom,
  uniqArr,
  waitForFunction,
  waitForSelector,
} from '../utils';
import { Scraper } from './scraper';

export type IMDBFilm = {
  typeName: 'IMDBFilm';
  title: string;
  coverURL: string;
  releaseDate: string;
  originalLanguage: string;
  countriesOfOrigin: string;
  creators: string;
  cast: string;
  seasons?: number;
  episodes?: number;
  duration: string;
  description: string;
};

// In general, its hard to scrape data from IMDB because page structure / information order
// often changes based on content type (movie/series/short series etc.)

export class IMDBFilmScraper extends Scraper<'IMDBFilm', IMDBFilm> {
  // https://www.imdb.com/title/tt0133093/
  readonly pattern = new URLPattern({
    hostname: 'www.imdb.com',
    pathname: '/title/*',
  });

  readonly scrape = async (): Promise<IMDBFilm> => {
    scrollToBottom();

    await waitForFunction(
      () => document.querySelector('[data-testid=storyline-loader]') === null,
      'wait until loader is gone'
    );

    const metadata = getListValues(document, 'h1[data-testid=hero__pageTitle]~.ipc-inline-list li');
    while (metadata.length < 4) {
      metadata.unshift('');
    }

    const filmType = metadata[0].toLowerCase();
    const isSeries = filmType.includes('series');
    const isMiniSeries = isSeries && filmType.includes('mini');

    const title = (await waitForSelector(document, 'h1[data-testid=hero__pageTitle]', 'title'))
      .innerText;
    const originalLanguage = getEl(
      '[data-testid=title-details-languages] ul li a',
      'original language'
    ).innerText;
    const countriesOfOrigin = getListStr(document, '[data-testid=title-details-origin] ul li a');

    const coverURL = getEl<HTMLImageElement>(
      '[data-testid=hero-media__poster] img',
      'cover image'
    ).src;

    const summaryEl = await waitForSelector(
      document,
      '[data-testid=storyline-plot-summary]',
      'description'
    );
    // cleanup description if needed - remove nickname
    getEl(summaryEl, 'div>span')?.remove();

    const description = getEl(summaryEl, 'div', 'description').innerText;

    const releaseDate = metadata[1];
    let duration = metadata[3];

    const creators = [];
    const cast = [];
    const creditsEls = getAll(document, '[data-testid=title-pc-principal-credit]');

    let seasons: number | undefined = undefined;
    let episodes: number | undefined = undefined;
    if (isSeries) {
      seasons = Number.parseInt(
        getEl(
          '[data-testid=episodes-browse-episodes] >:nth-child(2) >:nth-child(2)',
          'seasons count'
        ).innerText,
        10
      );
      episodes = Number.parseInt(
        getEl('[data-testid=episodes-header] .ipc-title__subtext', 'episodes count').innerText,
        10
      );

      creators.push(...getListValues(creditsEls[0], ':scope ul li a'));
      cast.push(...getListValues(creditsEls[1], ':scope ul li a'));
    } else {
      creators.push(...getListValues(creditsEls[0], ':scope ul li a'));
      creators.push(...getListValues(creditsEls[1], ':scope ul li a'));

      cast.push(...getListValues(creditsEls[2], ':scope ul li a'));
    }

    if (isMiniSeries) {
      // IMDB often shows total duration for miniseries instead of episode duration
      duration = '';
    }

    const chips = getListValues(document, '.ipc-chip__text').map((item) => item.toLowerCase());

    const isAnime = chips.includes('anime');
    if (isAnime) {
      creators.length = 0;
      cast.length = 0;
    }

    const isAnimation = chips.includes('animation');
    if (isAnimation) {
      cast.length = 0;
    }

    return {
      typeName: 'IMDBFilm',
      title,
      coverURL,
      releaseDate,
      originalLanguage,
      countriesOfOrigin,
      creators: uniqArr(creators).join(', '),
      cast: uniqArr(cast).join(', '),
      seasons,
      episodes,
      duration,
      description,
    };
  };
}
