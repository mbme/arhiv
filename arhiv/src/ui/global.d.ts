import { ArhivUIConfig } from 'dto';
import { AppController } from 'controller';

declare global {
  interface Window {
    CONFIG: ArhivUIConfig;
    APP: AppController;
  }
}

export {};
