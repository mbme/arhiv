import { Scraper } from '../scraper';
import { getEl, getLocationURL, parseHumanDate, promiseTimeout } from '../utils';
import { isFB, isPostListPage } from './utils';

type PostListItem = {
  permalink: string;
  date: string;
  dateISO?: string;
  content: string;
};

export type FacebookPostList = PostListItem[];

const hasLoader = (el: HTMLElement) => !!el.querySelector('[role=progressbar]');
const isComment = (el: HTMLElement) => !!el.parentElement?.closest('[role=article]');

const clickSeeMore = (el: HTMLElement) => {
  const seeMoreBtn = Array.from(el.querySelectorAll<HTMLElement>('[role=button]')).find(
    (el) => el.textContent === 'See more'
  );

  if (!seeMoreBtn) {
    return;
  }

  seeMoreBtn.click();
};

export const scrapeFBPostList: Scraper<FacebookPostList> = async () => {
  const locationURL = getLocationURL();

  if (!isFB(locationURL) || !isPostListPage(locationURL)) {
    return undefined;
  }

  const postsElements = Array.from(document.querySelectorAll<HTMLElement>('[role=article]')).filter(
    (postEl) => !hasLoader(postEl) && !isComment(postEl)
  );

  postsElements.forEach((postEl) => {
    clickSeeMore(postEl);
  });
  await promiseTimeout(1000);

  const posts = postsElements.map((postEl) => {
    const dateEl = postEl.querySelectorAll('a')[3];
    if (!dateEl) {
      throw new Error("can't find date element");
    }

    const permalink = dateEl.href;
    const date = dateEl.innerText;
    const dateISO = parseHumanDate(date)?.toISOString();

    const content = getEl(postEl, '[data-ad-preview=message]', 'content element').innerText;

    return {
      permalink,
      date,
      dateISO,
      content,
    };
  });

  return posts;
};
