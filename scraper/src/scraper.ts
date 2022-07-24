export type Scraper<T = unknown> = () => Promise<T | undefined> | T | undefined;
