export abstract class Scraper<TypeName extends string, Data extends { typeName: TypeName }> {
  abstract canScrape(locationURL: URL): boolean;

  protected abstract _scrape: ((locationURL: URL) => Data) | ((locationURL: URL) => Promise<Data>);

  scrape(locationURL: URL): Promise<Data> | Data {
    if (!this.canScrape(locationURL)) {
      throw new Error(`can't scrape ${locationURL.toString()}`);
    }

    return this._scrape(locationURL);
  }
}

export type ExtractScraperGeneric<Type> = Type extends Scraper<string, infer X> ? X : never;
