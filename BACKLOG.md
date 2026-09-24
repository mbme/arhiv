# Backlog

- network p2p sync (use iroh), relays
- better diff/merge
- integration with emacs/file system - use FUSE?
- rethink "save" + "commit" distinction / approach? no need to "save"? + auto-commit

- better Auth/accounts
  - user key pair? maybe independent of iroh and age?
  - support multiple devices, revocations etc.

- better UI
  - redesign interface?
    - can't reorder notes
    - multiple workspaces
  - need better ergonomics
  - no taxonomy, no way to browse documents
  - no way to list staged changes (diff)
  - need better way to see documents with conflicts, their diff & conflicts; also in CLI
  - no way to see document history
  - i don't like switching between edit/preview modes in the editor
  - improve password input: allow to see plain text
  - i'd like to improve collections UX
  - timeline view

- better integration
  - CLI API for LLMs
  - Skill for LLMs for using CLI
  - "arhiv json" format for paste/cli import?
  - FS integration - FUSE? would be great to edit with Emacs & others
  - externally scriptable

- better content management
  - dynamic schemas? 3rd party schema? rely on standards? markdown + yaml header + "gradual typing"?
  - I'd like to separate data schema from code
  - mark notes stale/irrelevant/archived
  - різні рівні "доступу" - враховувати при інтеграціях і публікаціях - приватне, ДСК і публічне
  - temporary/scratchpad documents? (without long history)
  - better search/filtering using expression engine (cel-rust or wirefilter)
  - smart folders/saved searches?

- як це розкладаєтьс на "базові" компоненти? і інтегрується із рештою екосистеми?
  - формат даних (encrypted compressed jsonl)
  - проста база даних із схемою, гілками і мерджем, eventual consistency & conflict resolution
  - file management
  - p2p sync
  - arhiv app
  - storage/(read/write APIs i.e. FS) for other apps
- single folder mode? keep state in "syncable dir"?
- refactor: arhiv-cli shouldn't probably access baza directly
- remote backup without 3rd party tools - separate "backup manager"?
- browse logs in android app
- look up how mergiraf resolves merge conflicts for markdown
- do we delete data from search index when erased?

## Information Decay system

Modern version of Lifestreams: don't prematurely organize; capture, derive structure, continuously reassess salience, compress, and deliberately forget.

- Information should age from raw data toward increasingly compressed knowledge.
- storage lifetime ≠ visibility ≠ semantic importance.

```text
                 ┌→ pinned / permanent
item → lifecycle ├→ normal decay
                 ├→ summarize then delete raw data
                 └→ expire automatically
```