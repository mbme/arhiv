import { Scraper } from '../scraper';
import { getEl, getLocationURL, parseHumanDate } from '../utils';
import { isFB, isPostPage } from './utils';

export type FacebookPost = {
  permalink: string;
  date: string;
  dateISO?: string;
  content: string;
};

export const scrapeFBPost: Scraper<FacebookPost> = () => {
  const locationURL = getLocationURL();

  if (!isFB(locationURL) || !isPostPage(locationURL)) {
    return undefined;
  }

  const postEl = getEl(document, '[role=article]', 'post element');

  const content = getEl(postEl, '[data-testid=post_message]', 'post content').innerText;

  const dateEl = postEl.querySelectorAll('a')[3];
  if (!dateEl) {
    throw new Error("can't find date element");
  }

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
