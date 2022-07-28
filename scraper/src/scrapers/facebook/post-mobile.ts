import { Scraper } from '../scraper';
import { getEl, getSelectionString, parseHumanDate } from '../../utils';
import { isFBMobile, isPostPage } from './utils';

export type FacebookMobilePost = {
  permalink: string;
  date: string;
  dateISO?: string;
  content: string;
};

// https://m.facebook.com/theprodigyofficial/posts/pfbid0WoM5Kzm79yfeiBKqR9FkfhsVXA6CeqW4DtzJbyKnc56xw7kytdQYfqwgK55hoheFl
export const scrapeFBMobilePost: Scraper<FacebookMobilePost> = (locationURL) => {
  if (!isFBMobile(locationURL) || !isPostPage(locationURL)) {
    return undefined;
  }

  const postEl = getEl(document, '.story_body_container', 'post element');

  const content = getSelectionString(getEl(postEl, ':scope>div', 'post content'));

  const dateEl = getEl(postEl, 'header a abbr', 'post date').parentElement as HTMLAnchorElement;

  const permalink = dateEl.href;

  const date = dateEl.innerText;
  const dateISO = parseHumanDate(date)?.toISOString();

  return {
    permalink,
    date,
    dateISO,
    content,
  };
};
