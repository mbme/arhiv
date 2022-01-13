import { Context } from './context';
import { getListStr, getListValues, getText, Obj } from './utils';
import { uniqArr } from './utils';

// In general, its hard to scrape data from IMDB because page structure / information order
// often changes based on content type (movie/series/short series etc.)
export async function extractFilmFromIMDB(url: string, context: Context): Promise<boolean> {
  // https://www.imdb.com/title/tt0133093/
  if (!url.includes('imdb.com/title/')) {
    return false;
  }

  const page = await context.newPage(url);

  const metadata = await getListValues(page, '[data-testid=hero-title-block__metadata] li');

  const isSeries = metadata[0].toLowerCase().includes('series');
  const isMiniSeries = isSeries && metadata[0].toLowerCase().includes('mini');

  const data: Obj = {
    title: await getText(page, 'h1[data-testid=hero-title-block__title]'),
    original_language: await getText(page, '[data-testid=title-details-languages] ul li a'),
    countries_of_origin: await getListStr(page, '[data-testid=title-details-origin] ul li a'),
    is_series: isSeries,
  };

  const cover_src  = await page.$eval('[data-testid=hero-media__poster] img', node => (node as HTMLImageElement).src);
  data.cover = await context.channel.createAttachment(cover_src);

  let description = '';
  const summaryEl = await page.$('[data-testid=storyline-plot-summary]');
  if (summaryEl) {
    // cleanup description if needed - remove nickname
    await summaryEl.$$('div>span').then((nodes) => {
      if (!nodes.length) {
        return;
      }

      return nodes[0].evaluate(node => node.remove());
    });

    description = await getText(summaryEl, 'div');
  }
  data['description'] = description;


  const creators = [];
  const cast = [];
  const creditsEls = await page.$$('[data-testid=title-pc-principal-credit]');

  if (isSeries) {
    data['release_date'] = metadata[1];
    data['number_of_episodes'] = Number.parseInt(await getText(page, '[data-testid=hero-subnav-bar-series-episode-count]'), 10);
    data['episode_duration'] = metadata[3];

    creators.push(...(await getListValues(creditsEls[0], ':scope ul li a')));
    cast.push(...(await getListValues(creditsEls[1], ':scope ul li a')));

    if (isMiniSeries) { // IMDB often shows total duration for miniseries instead of episode duration
      data['episode_duration'] = undefined;
    }
  } else {
    data['release_date'] = metadata[0];
    data['duration'] = metadata[2];

    creators.push(...(await getListValues(creditsEls[0], ':scope ul li a')));
    creators.push(...(await getListValues(creditsEls[1], ':scope ul li a')));

    cast.push(...(await getListValues(creditsEls[2], ':scope ul li a')));
  }

  data['creators'] = uniqArr(creators).join(', ');
  data['cast'] = uniqArr(cast).join(', ');

  await context.channel.createDocument('film', data);

  await page.close();

  return true;
}
