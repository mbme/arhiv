import { Scraper } from '../scraper';
import { getEl, getSelectionString, parseHumanDate } from '../../utils';
import { isFBMobile, isPostPage } from './utils';

export type FacebookMobilePost = {
  typeName: 'FacebookMobilePost';
  permalink: string;
  date: string;
  dateISO?: string;
  content: string;
};

export class FBMobilePostScraper extends Scraper<'FacebookMobilePost', FacebookMobilePost> {
  canScrape(locationURL: URL): boolean {
    // https://m.facebook.com/theprodigyofficial/posts/pfbid0WoM5Kzm79yfeiBKqR9FkfhsVXA6CeqW4DtzJbyKnc56xw7kytdQYfqwgK55hoheFl
    return isFBMobile(locationURL) && isPostPage(locationURL);
  }

  protected _scrape = (): FacebookMobilePost => {
    const postEl = getEl('.story_body_container', 'post element');

    const content = getSelectionString(getEl(postEl, ':scope>div', 'post content'));

    const dateEl = getEl(postEl, 'header a abbr', 'post date').parentElement as HTMLAnchorElement;

    const permalink = dateEl.href;

    const date = dateEl.innerText;
    const dateISO = parseHumanDate(date)?.toISOString();

    return {
      typeName: 'FacebookMobilePost',
      permalink,
      date,
      dateISO,
      content,
    };
  };
}
