import { Context } from './context';
import { getImageSrc, getTable, getText, Obj } from './utils';

export async function extractFilmFromMyanimelist(url: string, context: Context): Promise<boolean> {
  // https://myanimelist.net/anime/30276/One_Punch_Man
  if (!url.includes('myanimelist.net/anime/')) {
    return false;
  }

  const page = await context.newPage(url);

  const data: Obj = {};

  const engTitle = await getText(page, '.title-english').catch(() => '');
  data.title = engTitle || (await getText(page, '.title-name'));

  const cover_src = await getImageSrc(page, '.leftside img');
  data.cover = await context.channel.createAttachment(cover_src);

  const metadata = await getTable(page, '.leftside .spaceit_pad');

  data.release_date = metadata.Aired;
  data.original_language = 'Japanese';
  data.countries_of_origin = 'Japan';
  data.creators = metadata.Studios;
  data.duration = metadata.Duration;
  data.description = await getText(page, '[itemprop=description]');

  const related = await getTable(page, '.anime_detail_related_anime tr');
  if (related.Prequel) {
    throw new Error("Can't import an anime: it has a prequel. Start with the first season.");
  }

  await context.channel.createDocument('film', '', data);

  return true;
}
