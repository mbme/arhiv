import { Scraper } from '../scraper';
import { getAll, getEl, getSelectionString, parseHumanDate } from '../../utils';
import { isFB, isPostPage } from './utils';

export type FacebookPost = {
  permalink: string;
  date: string;
  dateISO?: string;
  content: string;
  images: string[];
};

// https://www.facebook.com/theprodigyofficial/posts/pfbid0WoM5Kzm79yfeiBKqR9FkfhsVXA6CeqW4DtzJbyKnc56xw7kytdQYfqwgK55hoheFl
export const scrapeFBPost: Scraper<FacebookPost> = (locationURL) => {
  if (!isFB(locationURL) || !isPostPage(locationURL)) {
    return undefined;
  }

  const postEl = getEl('[role=article]', 'post element');

  const content = getSelectionString(getEl(postEl, '[data-testid=post_message]', 'post content'));

  const dateEl = postEl.querySelector('a > abbr')?.parentElement;
  if (!dateEl) {
    throw new Error("can't find date element");
  }

  const permalink = (dateEl as HTMLAnchorElement).href;

  const date = dateEl.innerText;
  const dateISO = parseHumanDate(date)?.toISOString();

  const images = getAll<HTMLImageElement>(postEl, '[data-testid=post_message] ~ div a img').map(
    (img) => img.src
  );

  return {
    permalink,
    date,
    dateISO,
    content,
    images,
  };
};
