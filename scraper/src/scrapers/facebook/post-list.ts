import { Scraper } from '../scraper';
import { getEl, getAll, parseHumanDate, promiseTimeout, getSelectionString } from '../../utils';
import { isFB, isPostListPage } from './utils';

type PostListItem = {
  permalink: string;
  date: string;
  dateISO?: string;
  content: string;
  images: string[];
  videos: string[];
};

export type FacebookPostList = PostListItem[];

const hasLoader = (el: HTMLElement) => !!el.querySelector('[role=progressbar]');

const isInCommentSection = (el: HTMLElement) =>
  Boolean(el.parentElement?.closest('[data-visualcompletion="ignore-dynamic"]'));

const clickSeeMore = (el: HTMLElement) => {
  const seeMoreBtn = getAll(el, '[role=button]').find((el) => el.textContent === 'See more');

  if (!seeMoreBtn) {
    return;
  }

  seeMoreBtn.click();
};

const collectLinks = (postEl: HTMLElement): HTMLAnchorElement[] =>
  getAll<HTMLAnchorElement>(postEl, 'a').filter((link) => !isInCommentSection(link));

const collectImages = (postEl: HTMLElement): HTMLImageElement[] =>
  getAll<HTMLImageElement>(postEl, 'a img').filter((img) => !isInCommentSection(img));

const collectVideos = (postEl: HTMLElement): HTMLVideoElement[] =>
  getAll<HTMLVideoElement>(postEl, 'video').filter((img) => !isInCommentSection(img));

export const scrapeFBPostList: Scraper<FacebookPostList> = async (locationURL) => {
  if (!isFB(locationURL) || !isPostListPage(locationURL)) {
    return undefined;
  }

  const postsElements = getAll(document, '[role=article]').filter(
    (postEl) => !hasLoader(postEl) && !isInCommentSection(postEl)
  );
  console.log(`scraping ${postsElements.length} posts`);

  const posts: FacebookPostList = [];
  for (let i = 0; i < postsElements.length; i += 1) {
    const postEl = postsElements[i];

    postEl.scrollIntoView(true);
    clickSeeMore(postEl);
    await promiseTimeout(1000);

    const links = collectLinks(postEl);
    const dateEl = links[3];
    if (!dateEl) {
      throw new Error("can't find date element");
    }

    const permalink = dateEl.href;
    const date = dateEl.innerText;
    const dateISO = parseHumanDate(date)?.toISOString();

    const content = getSelectionString(
      getEl(postEl, '[data-ad-preview=message]', 'content element')
    );

    const images = collectImages(postEl).map((img) => img.src);

    const videos = collectVideos(postEl).map((video) => video.src);

    posts.push({
      permalink,
      date,
      dateISO,
      content,
      images,
      videos,
    });

    console.log(`Scraped post ${i + 1} of ${postsElements.length}`);
  }

  posts.reverse();

  return posts;
};
