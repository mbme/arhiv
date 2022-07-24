import { Scraper } from '../scraper';
import { getEl, getLocationURL, parseHumanDate } from '../utils';
import { FacebookPostList } from './post-list';
import { isFBMobile, isPostListPage } from './utils';

export const scrapeFBMobilePostList: Scraper<FacebookPostList> = () => {
  const locationURL = getLocationURL();

  if (!isFBMobile(locationURL) || !isPostListPage(locationURL)) {
    return undefined;
  }

  const posts = Array.from(document.querySelectorAll<HTMLElement>('article')).map((postEl) => {
    const dateEl = getEl(postEl, 'header a abbr', 'post date').parentElement as HTMLAnchorElement;

    const permalink = dateEl.href;
    const date = dateEl.innerText;
    const dateISO = parseHumanDate(date)?.toISOString();

    const preview = getEl(postEl, 'header~div', 'preview element').innerText;

    return {
      permalink,
      date,
      dateISO,
      preview,
    };
  });

  return {
    posts,
  };
};
