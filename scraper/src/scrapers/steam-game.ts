import { getEl, getListStr, getPathSegments } from '../utils';
import { Scraper } from './scraper';

type SteamGame = {
  coverURL: string;
  name: string;
  releaseDate: string;
  developers: string;
  description: string;
};

// https://store.steampowered.com/app/814380/Sekiro_Shadows_Die_Twice__GOTY_Edition/
export const scrapeGameFromSteam: Scraper<SteamGame> = (locationURL) => {
  const pathSegments = getPathSegments(locationURL);
  if (locationURL.hostname !== 'store.steampowered.com' || pathSegments[0] !== 'app') {
    return undefined;
  }

  const coverURL = getEl<HTMLImageElement>('.game_header_image_full', 'cover image').src;
  const name = getEl('#appHubAppName', 'game name').innerText;
  const releaseDate = getEl('.release_date .date', 'release date').innerText;
  const developers = getListStr(document, '#developers_list a');

  // remove "About this game"
  getEl('#game_area_description h2', 'description header').remove();

  const description = getEl('#game_area_description', 'description').innerText;

  return {
    coverURL,
    name,
    releaseDate,
    developers,
    description,
  };
};
