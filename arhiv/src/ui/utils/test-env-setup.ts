import 'global-jsdom/register';
global.Event = window.Event; // override nodejs built-in
global.FormData = window.FormData; // override nodejs built-in

window.FEATURES = {
  use_local_storage: false,
};
