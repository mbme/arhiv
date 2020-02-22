# What is arhiv
* plain text database
* tool to manage (create/edit/query/search...), sync, backup documents (including large media files)
* universal, for any kind of information
* local first
* holistic
* data format
* cross-linking

# Why
* platform to integrate all the data
* decreasing complexity
* controlling data

# Characteristics
* single-user database
* isodb stores documents and attachments, which share id namespace
* attachments have BLOBs
* database has revisions 
  - empty database has revision 0
  - primary increases revision on each update

# Primary arhiv server
* the main instance of isodb with all the data (even giant attachments like movies)
* there might be only one primary instance at a time
  - lock file to allow only a single instance to read/write the db
* primary receive data from replicas in a form of Changesets
* primary doesn't create data
* single queue (kind of "serializable" transaction isolation level)
* primary manages revision 
* file structure
  - /documents/:id/:rev 
  - /attachments/:id/data
  - /attachments/:id/metadata
  - /settings
  - /lock

# Replica
* app-local instance of the isodb which gets its data from the primary
* replica has a full copy of documents and attachments (without BLOBs)
* replica choose which BLOBs to fetch from primary
* replica allows to create/update/delete documents and create attachments
* replica maintains document write locks, global db lock and global leader lock
  - locks are synced across multiple tabs
  - db lock means no write lock can be acquired
  - db lock can't be acquired while there are write locks
  - leader lock allows tab to schedule regular db sync
* replica runs compaction when there are 0 locks
* replica has reactive api for documents
* replace doesn't have reactive api for attachments
  - cause attachments would always be available if documents are available, and they are read-only, so they would never update on-the-fly

# What's a document
object with 
* id
 - random string
 - replica generates ids
* rev
 - non-negative integer
 - primary manages rev and updates it on document create/update
* createdAt 
 - ISO8601 date string
* updatedAt
 - ISO8601 date string
*  attachmentRefs
 - attachment ids
* refs
 - ids of other documents
* deleted
 - boolean
* may contain other fields too
 - in case of _deleted: true only these basic fields would be present

* create document
 - replicate creates local document with random id and without revision
 - optionally, document may reference existing attachments
 - sync

* update document
 - replica creates local document with the same id but without revision
 - optionally, document may reference existing attachments
 - sync

* delete document
 - replica updates document: removes all fields except basic one and sets _deleted: true
 - sync

# What's an attachment
* object with 
 - id
   - random string
   - replica generates ids
 - rev
   - non-negative integer
   - primary manages rev and updates it on attachment update
 - createdAt 
   - ISO8601 date string
 - mimeType (mime-type)
   - primary manages mimeType and updates it on attachment create, may update later
 - size
   - number of bytes
* attachments are "immutable" for replicas
* attachment have a BLOB, attachment id === BLOB id
* create attachment
  - replicate creates local attachment with random id and without revision
  - replica attaches new blob
  - sync
  - primary should make sure _mimeType and _size are correct or update them otherwise
* primary only: update attachment
  - happens only on primary in rare cases when mime type should be updated
  - revision should be increased
* primary only: delete attachment
  - happens automatically on primary
  - primary updates attachment: _deleted: true
  - primary deletes blob
  - revision should be increased

# Sync protocol
* replica sends Changeset
  - baseRev - replica rev
  - documents[] - new or updated documents
  - attachments[] - new attachments
* primary returns ChangesetResponse
  - status is 'accepted' or 'outdated'
    - 'accepted' means replica's changes were accepted and saved (cause there were no merge conflicts)
    - 'outdated' means replica's changes were declined due to merge conflicts
* replica 
  - if no merge conflicts
    - send a changeset with revision, new documents, attachments and blobs
      - ignore unused local attachments
    - primary responds with new revision, new documents and attachments
    - merge response with local data
* primary
  - if changeset rev < primary rev then send documents and attachments with rev > changeset rev
  - if changeset rev > primary rev then throw an error
  - if changeset rev is equal to primary rev 
    - if changeset contains no documents 
      - then 
        - send empty response
      - else 
        - increase revision
        - process new documents and attachments
          - merge if needed
          - set increased revision on them
          - save blobs
        - send documents and attachments with rev > changeset rev

# How merge works
* merge should be always performed by replicas
* merge may produce merge conflicts, which should be resolved externally by the app
* documents
  - if replica got new document from the primary
    - add new document to local db
  - if replica got new revision of the document from the primary
    - if document was modified locally
      - document was modified locally and remotely
        - merge conflict
      - document was modified locally and _deleted remotely
        - merge conflict
      - document was _deleted locally and modified remotely
        - merge conflict
      - document was _deleted locally and remotely
        - save primary revision into local db
      - local document references _deleted attachment
        - warn
    - else 
      - save primary revision into local db
* attachments
  - replace local attachments with primary attachments (cause attachments are "immutable" for replicas)

# How compaction works
* on primary
  - (should be triggered manually)
  - group document changes by period of time (like 1 per hour or 1 per day)
  - for all attachments not referenced by active documents
    - mark them as _deleted
    - increase revision
    - delete their blobs
  - (active documents are latest revisions of documents and aren't _deleted)
* on replica
  - on startup
  - remove "deleted" local new documents
  - remove "unchanged" local documents (documents which are equal to their "base" version)
  - remove unused local (new) attachments 

# Versioning
* isodb keeps previous versions of documents
  - primary saves previous version of a document when a new version arrives
  - primary allows to query previous versions of the document
  - replica doesn't keep previous versions of documents
    - cause merge process would be very complicated
* isodb doesn't keep previous versions of attachments or their metadata

# Backups
* daily, weekly, monthly (oldest backup is one month old)
* rsync incremental backup over local network on usb hdd
* replicas would be on a different devices in different places, so they are kind of "backup" backups
* run integrity checks of all snapshots after backup
* restore strategy
  - an option to restore from any of the incremental backups
  - an option to save replica on the file system
* after restore, apply schema migration if needed

# ? schema migration
* primary to have "info" file with current schema version (and probably latest revision etc)
* migration scenarios (functions) on primary
* schema version is included in each request
* ? if replica sends new data with the old schema, primary updates data using migration scenarios and replica reloads all the documents
  - what if replica is outdated & has local changes?
* attachments schema never change
* backups shouldn't be migrated, migration scenarios should be automatically applied after restore
* previous versions of documents should be migrated
* handle SCHEMA_VERSION

# Document types
* note
* task
* log
* events/calendar/reminders
  - put events into logs
* track
* movie
* photo
* picture
* book
* game (+ blobs)
* software
* bookmark
* snippet
* inventory item
* list
  - acts as a playlist for tracks/movies, topic for tasks
