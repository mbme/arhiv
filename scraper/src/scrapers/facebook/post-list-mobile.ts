import { Scraper } from '../scraper';
import { getAll, getEl, parseHumanDate } from '../../utils';

type PostListItem = {
  permalink: string;
  date: string;
  dateISO?: string;
  preview: string;
};

export type FacebookMobilePostList = {
  typeName: 'FacebookMobilePostList';
  posts: PostListItem[];
};

export class FBMobilePostListScraper extends Scraper<
  'FacebookMobilePostList',
  FacebookMobilePostList
> {
  // https://m.facebook.com/vmistozher/
  pattern = new URLPattern({
    hostname: 'm.facebook.com',
    pathname: '/:group/',
  });

  readonly scrape = (): FacebookMobilePostList => {
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

    return {
      typeName: 'FacebookMobilePostList',
      posts,
    };
  };
}
