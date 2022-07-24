export type Scraper<T> = () => Promise<T | undefined> | T | undefined;

export type ExtractScraperGeneric<Type> = Type extends Scraper<infer X> ? X : never
