import { Page } from 'puppeteer-core';
import { intoFacebookMobileURL, isFB, isLoginPage, isMobileLoginPage } from './utils';

export async function handleFacebookRedirect(page: Page, originalURLStr: string): Promise<string | undefined> {
  const originalURL = new URL(originalURLStr);

  if (!isFB(originalURL)) {
    return undefined;
  }

  const currentURLStr = await page.evaluate(() => window.location.href);
  const currentURL = new URL(currentURLStr);

  if (isLoginPage(currentURL) && !isLoginPage(originalURL) && !isMobileLoginPage(originalURL)) {
    // we got redirected to the login page, try opening the same url in mobile mode to avoid the redirect

    console.error('Got redirected to Facebook login page, trying mobile mode');

    const mobileURL = intoFacebookMobileURL(originalURL);

    return mobileURL.toString();
  }

  return undefined
}
