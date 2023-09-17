import { DataSchema } from 'utils/schema';

declare global {
  declare var API_ENDPOINT: string;
  declare var SCHEMA: DataSchema;
}
