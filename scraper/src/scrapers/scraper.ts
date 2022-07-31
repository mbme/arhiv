import 'urlpattern-polyfill';

export abstract class Scraper<TypeName extends string, Data extends { typeName: TypeName }> {
  abstract readonly pattern: URLPattern;

  abstract readonly scrape: (() => Data) | (() => Promise<Data>);
}

export type ExtractScraperGeneric<Type> = Type extends Scraper<string, infer X> ? X : never;
