import { getPathSegments } from '../../utils';

export const isFB = (url: URL) => url.hostname === 'www.facebook.com';
export const isLoginPage = (url: URL) => isFB(url) && url.pathname === '/login/';

export const isFBMobile = (url: URL) => url.hostname === 'm.facebook.com';
export const isMobileLoginPage = (url: URL) => isFBMobile(url) && url.pathname === '/login.php';

export const isPostPage = (url: URL) => getPathSegments(url)[1] === 'posts';

export const isPostListPage = (url: URL) => getPathSegments(url).length === 1;

export const intoFacebookMobileURL = (url: URL): string => {
  url.host = 'm.facebook.com';

  return url.toString();
};
