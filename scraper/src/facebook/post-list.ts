import { Scraper } from '../scraper';
import { getEl, getLocationURL, parseHumanDate } from '../utils';
import { isFB, isPostListPage } from './utils';

type PostListItem = {
  permalink: string;
  date: string;
  dateISO?: string;
  preview: string;
};
export type FacebookPostList = {
  posts: PostListItem[];
};

const hasLoader = (el: HTMLElement) => !!el.querySelector('[role=progressbar]');
const isComment = (el: HTMLElement) => !!el.parentElement?.closest('[role=article]');

export const scrapeFBPostList: Scraper<FacebookPostList> = () => {
  const locationURL = getLocationURL();

  if (!isFB(locationURL) || !isPostListPage(locationURL)) {
    return undefined;
  }

  const posts = Array.from(document.querySelectorAll<HTMLElement>('[role=article]'))
    .filter((postEl) => !hasLoader(postEl) && !isComment(postEl))
    .map((postEl) => {
      const dateEl = postEl.querySelectorAll('a')[3];
      if (!dateEl) {
        throw new Error("can't find date element");
      }

      const permalink = dateEl.href;
      const date = dateEl.innerText;
      const dateISO = parseHumanDate(date)?.toISOString();

      const preview = getEl(postEl, '[data-ad-preview=message]', 'preview element').innerText;

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
