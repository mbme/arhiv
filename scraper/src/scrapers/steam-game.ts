import { getEl, getListStr } from '../utils';
import { Scraper } from './scraper';

type SteamGame = {
  typeName: 'SteamGame';
  coverURL: string;
  name: string;
  releaseDate: string;
  developers: string;
  description: string;
};

export class SteamGameScraper extends Scraper<'SteamGame', SteamGame> {
  // https://store.steampowered.com/app/814380/Sekiro_Shadows_Die_Twice__GOTY_Edition/
  readonly pattern = new URLPattern({
    hostname: 'store.steampowered.com',
    pathname: '/app/*',
  });

  readonly scrape = (): SteamGame => {
    const coverURL = getEl<HTMLImageElement>('.game_header_image_full', 'cover image').src;
    const name = getEl('#appHubAppName', 'game name').innerText;
    const releaseDate = getEl('.release_date .date', 'release date').innerText;
    const developers = getListStr(document, '#developers_list a');

    // remove "About this game"
    getEl('#game_area_description h2', 'description header').remove();

    const description = getEl('#game_area_description', 'description').innerText;

    return {
      typeName: 'SteamGame',
      coverURL,
      name,
      releaseDate,
      developers,
      description,
    };
  };
}
