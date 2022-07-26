import { Scraper } from '../scraper';
import { getAll, getEl, getLocationURL, parseHumanDate } from '../utils';
import { isFBMobile, isPostListPage } from './utils';

type PostListItem = {
  permalink: string;
  date: string;
  dateISO?: string;
  preview: string;
};

export type FacebookMobilePostList = PostListItem[];

export const scrapeFBMobilePostList: Scraper<FacebookMobilePostList> = () => {
  const locationURL = getLocationURL();

  if (!isFBMobile(locationURL) || !isPostListPage(locationURL)) {
    return undefined;
  }

  const posts = getAll(document, 'article').map((postEl) => {
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

  posts.reverse();

  return posts;
};
