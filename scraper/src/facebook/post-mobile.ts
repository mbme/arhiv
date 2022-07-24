import { Scraper } from '../scraper';
import { getEl, getLocationURL, parseHumanDate } from '../utils';
import { Post } from './post';
import { isFBMobile, isPostPage } from './utils';

export const scrapeFBMobilePost: Scraper<Post> = () => {
  const locationURL = getLocationURL();

  if (!isFBMobile(locationURL) || !isPostPage(locationURL)) {
    return undefined;
  }

  const postEl = getEl(document, '.story_body_container', 'post element');

  const content = getEl(postEl, ':scope>div', 'post content').innerText;

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
